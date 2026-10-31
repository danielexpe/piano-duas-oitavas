#!/usr/bin/env bash
set -uo pipefail

APP_NAME="piano-duas-oitavas"
OUT="${OUT:-testes_debug.txt}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
TMP_DIR="$(mktemp -d)"
APP_BIN="${APP_BIN:-}"

section() { printf '\n\n===== %s =====\n' "$*"; }

run() {
    printf '\n$ %s\n' "$*"
    "$@" 2>&1
    printf '[exit %s]\n' "$?"
}

restore_console() {
    if command -v python3 >/dev/null 2>&1; then
        python3 - >/dev/null 2>&1 <<'PY' || true
import fcntl
import os

try:
    fd = os.open("/dev/tty", os.O_RDWR | os.O_NOCTTY)
    try:
        try:
            fcntl.ioctl(fd, 0x4B3A, 0)  # KDSETMODE, KD_TEXT
        except OSError:
            pass
        os.write(fd, b"\x1b[0m\x1b[?25h\x1b[2J\x1b[H")
    finally:
        os.close(fd)
except OSError:
    pass
PY
    else
        printf '\033[0m\033[?25h' >/dev/tty 2>/dev/null || true
    fi
}

cleanup() {
    local status=$?
    trap - EXIT
    restore_console
    rm -rf "$TMP_DIR"
    exit "$status"
}

trap 'echo "Interrompido por sinal."; exit 130' INT TERM HUP
trap cleanup EXIT

printf 'Coletando diagnóstico em %s\n' "$OUT" >/dev/tty 2>/dev/null || true
exec >"$OUT" 2>&1

find_app() {
    if [ -n "$APP_BIN" ] && [ -x "$APP_BIN" ]; then
        printf '%s\n' "$APP_BIN"
        return 0
    fi
    if command -v "$APP_NAME" >/dev/null 2>&1; then
        command -v "$APP_NAME"
        return 0
    fi
    for candidate in \
        "$SCRIPT_DIR/$APP_NAME" \
        "$PWD/$APP_NAME" \
        "$SCRIPT_DIR/target/release/$APP_NAME" \
        "$PWD/target/release/$APP_NAME"
    do
        [ -x "$candidate" ] && { printf '%s\n' "$candidate"; return 0; }
    done
    return 1
}

blank_state() {
    printf '\n--- blank/DPMS atual ---\n'
    for f in /sys/class/graphics/fb*/blank; do
        [ -r "$f" ] && printf '%s: %s\n' "$f" "$(cat "$f")"
    done 2>/dev/null || true
}

APP="$(find_app || true)"
TTY_NAME="$(tty 2>/dev/null || true)"
case "$TTY_NAME" in
    /dev/tty[0-9]*) VT_OK=1 ;;
    *) VT_OK=0 ;;
esac

section "Cabeçalho"
printf 'data: %s\n' "$(date -Is 2>/dev/null || date)"
printf 'hostname: %s\n' "$(hostname 2>/dev/null)"
printf 'kernel: %s\n' "$(uname -a)"
printf 'app: %s\n' "${APP:-NÃO ENCONTRADO}"
printf 'tty: %s (VT de texto: %s)\n' "${TTY_NAME:-?}" "$([ "$VT_OK" -eq 1 ] && echo sim || echo NÃO)"
if [ "$VT_OK" -eq 0 ]; then
    printf '\nAVISO: rode este script em um VT de texto (/dev/ttyN).\n'
    printf 'Em X/Wayland/SSH o teste não exercita o framebuffer e pode dar falso negativo.\n'
fi

section "Sistema"
run cat /etc/os-release
run uname -m
run cat /proc/cmdline

section "Usuário, sessão e terminal"
run id
run sh -c 'id -nG'
run sh -c 'printf "tty=%s\nTERM=%s\nXDG_SESSION_TYPE=%s\nDISPLAY=%s\nWAYLAND_DISPLAY=%s\n" "$(tty 2>/dev/null)" "${TERM:-}" "${XDG_SESSION_TYPE:-}" "${DISPLAY:-}" "${WAYLAND_DISPLAY:-}"'
run sh -c 'cat /sys/class/tty/tty0/active 2>/dev/null || true'
run sh -c 'loginctl 2>/dev/null | head -20 || true'

section "Permissões do framebuffer"
run ls -l /dev/fb0 /dev/fb1 /dev/fb2 2>/dev/null
run sh -c 'command -v udevadm >/dev/null && udevadm info --query=all --name=/dev/fb0 2>/dev/null || true'

section "Framebuffers do kernel"
run cat /proc/fb
run sh -c 'ls -l /dev/fb* 2>/dev/null || true'
run sh -c 'for d in /sys/class/graphics/fb*; do [ -d "$d" ] || continue; echo "--- $d"; for f in name virtual_size bits_per_pixel stride blank modes pan rotate; do [ -r "$d/$f" ] && echo "$f: $(cat "$d/$f")"; done; readlink -f "$d/device/driver" 2>/dev/null; done'

section "fbset"
run sh -c 'command -v fbset >/dev/null && fbset -i || echo "fbset ausente"'

section "Console virtual, fbcon e DRM"
run sh -c 'for f in /sys/class/vtconsole/vtcon*/name; do [ -r "$f" ] && echo "$f: $(cat "$f")"; done'
run sh -c 'ls -l /dev/dri 2>/dev/null || true'
run sh -c 'for d in /sys/class/drm/card*; do [ -e "$d/device/driver" ] && echo "$d -> $(readlink -f "$d/device/driver")"; done 2>/dev/null || true'
run sh -c 'lsmod 2>/dev/null | grep -Ei "fb|drm|amdgpu|i915|nouveau|simpledrm|efifb" || true'

section "Estado de blank/DPMS"
blank_state

section "App"
if [ -n "$APP" ]; then
    run "$APP" --version
    run file "$APP"
    run sh -c "ldd '$APP' 2>/dev/null || true"
else
    printf 'binário não encontrado (procurei %s no PATH e em target/release)\n' "$APP_NAME"
fi

section "App real sob strace (10 s)"
blank_state
if [ -n "$APP" ] && [ "$VT_OK" -eq 1 ]; then
    app_out="$TMP_DIR/app.out"
    strace_log="$TMP_DIR/strace.log"
    if command -v strace >/dev/null 2>&1 && strace -o /dev/null true >/dev/null 2>&1; then
        if command -v timeout >/dev/null 2>&1; then
            strace -f -tt -e trace=ioctl,openat,mmap,munmap,write -o "$strace_log" \
                timeout --signal=TERM --kill-after=5 10 "$APP" --backend fb >"$app_out" 2>&1
            rc=$?
        else
            strace -f -tt -e trace=ioctl,openat,mmap,munmap,write -o "$strace_log" \
                "$APP" --backend fb >"$app_out" 2>&1 &
            pid=$!
            sleep 10
            kill -TERM "$pid" 2>/dev/null || true
            wait "$pid"
            rc=$?
        fi
        printf 'exit=%s (124 = timeout esperado)\n' "$rc"
        printf '\n--- stdout/stderr do app ---\n'; cat "$app_out"
        printf '\n--- strace filtrado ---\n'
        grep -nE 'fb0|/dev/tty|ioctl|mmap|munmap|KDSET|KDGET|FBIOPAN|FBIOBLANK|FBIOGET|0x4b3|0x460|0x461' "$strace_log" | tail -400 || true
    else
        printf 'strace ausente ou sem permissão; teste do app ignorado\n'
    fi
else
    printf 'app ausente ou fora de VT; teste do app ignorado\n'
fi
printf '\n--- blank/DPMS depois do app ---\n'
blank_state

section "Teste de escrita direta (faixas RGB)"
if [ "$VT_OK" -eq 1 ] && command -v python3 >/dev/null 2>&1; then
    python3 - <<'PY'
import ctypes, fcntl, mmap, os, struct, time

FBIOGET_VSCREENINFO = 0x4600
FBIOGET_FSCREENINFO = 0x4602
FBIOPAN_DISPLAY = 0x4606
FBIOBLANK = 0x4611
FB_BLANK_UNBLANK = 0
KDGETMODE = 0x4B3B
KDSETMODE = 0x4B3A
KD_TEXT = 0
KD_GRAPHICS = 1

class Bitfield(ctypes.Structure):
    _fields_ = [("offset", ctypes.c_uint32),
                ("length", ctypes.c_uint32),
                ("msb_right", ctypes.c_uint32)]

class Var(ctypes.Structure):
    _fields_ = [("xres", ctypes.c_uint32), ("yres", ctypes.c_uint32),
                ("xres_virtual", ctypes.c_uint32), ("yres_virtual", ctypes.c_uint32),
                ("xoffset", ctypes.c_uint32), ("yoffset", ctypes.c_uint32),
                ("bits_per_pixel", ctypes.c_uint32), ("grayscale", ctypes.c_uint32),
                ("red", Bitfield), ("green", Bitfield), ("blue", Bitfield), ("transp", Bitfield),
                ("nonstd", ctypes.c_uint32), ("activate", ctypes.c_uint32),
                ("height", ctypes.c_uint32), ("width", ctypes.c_uint32),
                ("accel_flags", ctypes.c_uint32), ("pixclock", ctypes.c_uint32),
                ("left_margin", ctypes.c_uint32), ("right_margin", ctypes.c_uint32),
                ("upper_margin", ctypes.c_uint32), ("lower_margin", ctypes.c_uint32),
                ("hsync_len", ctypes.c_uint32), ("vsync_len", ctypes.c_uint32),
                ("sync", ctypes.c_uint32), ("vmode", ctypes.c_uint32),
                ("rotate", ctypes.c_uint32), ("colorspace", ctypes.c_uint32),
                ("reserved", ctypes.c_uint32 * 4)]
class Fix(ctypes.Structure):
    _fields_ = [("id", ctypes.c_char * 16), ("smem_start", ctypes.c_ulong),
                ("smem_len", ctypes.c_uint32), ("fb_type", ctypes.c_uint32),
                ("type_aux", ctypes.c_uint32), ("visual", ctypes.c_uint32),
                ("xpanstep", ctypes.c_uint16), ("ypanstep", ctypes.c_uint16),
                ("ywrapstep", ctypes.c_uint16), ("line_length", ctypes.c_uint32),
                ("mmio_start", ctypes.c_ulong), ("mmio_len", ctypes.c_uint32),
                ("accel", ctypes.c_uint32), ("capabilities", ctypes.c_uint16),
                ("reserved", ctypes.c_uint16 * 2)]

fd = ttyfd = mapping = None
mode = KD_TEXT
try:
    fd = os.open("/dev/fb0", os.O_RDWR | os.O_CLOEXEC)
    vbuf = bytearray(ctypes.sizeof(Var)); fcntl.ioctl(fd, FBIOGET_VSCREENINFO, vbuf, True)
    fbuf = bytearray(ctypes.sizeof(Fix)); fcntl.ioctl(fd, FBIOGET_FSCREENINFO, fbuf, True)
    var = Var.from_buffer(vbuf); fix = Fix.from_buffer(fbuf)
    print(f"var={var.xres}x{var.yres} virt={var.xres_virtual}x{var.yres_virtual} "
          f"yoffset={var.yoffset} bpp={var.bits_per_pixel}")
    print(f"line_length={fix.line_length} smem_len={fix.smem_len} ypanstep={fix.ypanstep}")
    print(f"bitfields R@{var.red.offset}/{var.red.length} G@{var.green.offset}/{var.green.length} "
          f"B@{var.blue.offset}/{var.blue.length} A@{var.transp.offset}/{var.transp.length}")

    ttyfd = os.open("/dev/tty", os.O_RDWR | os.O_NOCTTY)
    mbuf = bytearray(4); fcntl.ioctl(ttyfd, KDGETMODE, mbuf, True)
    mode = struct.unpack("i", mbuf)[0]; print(f"KDGETMODE antes: {mode}")

    try:
        fcntl.ioctl(fd, FBIOBLANK, FB_BLANK_UNBLANK); print("FBIOBLANK(unblank): ok")
    except OSError as error:
        print(f"FBIOBLANK(unblank): erro {error}")

    fcntl.ioctl(ttyfd, KDSETMODE, KD_GRAPHICS)
    os.write(ttyfd, b"\x1b[?25l\x1b[2J\x1b[H")

    bpp = var.bits_per_pixel // 8
    if bpp not in (2, 3, 4):
        raise RuntimeError(f"bpp não suportado: {var.bits_per_pixel}")
    stride = fix.line_length
    row_bytes = var.xres * bpp
    map_len = stride * (var.yoffset + var.yres)
    mapping = mmap.mmap(fd, map_len, flags=mmap.MAP_SHARED, prot=mmap.PROT_READ | mmap.PROT_WRITE)

    def ch(value, offset, length):
        return 0 if length == 0 else (((value * ((1 << length) - 1) + 127) // 255) << offset)
    def pack(r, g, b):
        return (ch(r, var.red.offset, var.red.length) | ch(g, var.green.offset, var.green.length)
                | ch(b, var.blue.offset, var.blue.length) | ch(255, var.transp.offset, var.transp.length))

    bands = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)]
    for y in range(var.yres):
        band = min(3, (y * 4) // var.yres)
        row = pack(*bands[band]).to_bytes(bpp, "little") * var.xres
        row += b"\x00" * (stride - row_bytes)
        start = (var.yoffset + y) * stride
        mapping[start:start + stride] = row
    print("faixas RGB escritas; aguardando 4 s...")
    time.sleep(4)
except Exception as error:
    print(f"teste de escrita falhou: {type(error).__name__}: {error}")
finally:
    try:
        if ttyfd is not None:
            fcntl.ioctl(ttyfd, KDSETMODE, KD_TEXT)
            os.write(ttyfd, b"\x1b[0m\x1b[?25h\x1b[2J\x1b[H")
    except OSError:
        pass
    try:
        if mapping is not None:
            mapping.close()
    except Exception:
        pass
    for handle in (fd, ttyfd):
        if handle is not None:
            os.close(handle)
PY
else
    printf 'teste de escrita ignorado (fora de VT ou sem python3)\n'
fi

section "Resumo"
printf 'arquivo: %s\n' "$OUT"
printf 'tamanho: %s bytes\n' "$(wc -c < "$OUT" 2>/dev/null || echo '?')"
printf 'linhas: %s\n' "$(wc -l < "$OUT" 2>/dev/null || echo '?')"

restore_console
printf '\nDiagnóstico concluído. Envie o arquivo: %s\n' "$OUT" >/dev/tty 2>/dev/null || true
