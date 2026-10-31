use std::ffi::CStr;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tiny_skia::Pixmap;

use crate::error::AppError;
use crate::i18n::{self, Key, Language};

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
const FBIOPAN_DISPLAY: libc::c_ulong = 0x4606;
const FBIOBLANK: libc::c_ulong = 0x4611;
const KDGETMODE: libc::c_ulong = 0x4B3B;
const KDSETMODE: libc::c_ulong = 0x4B3A;
const KD_TEXT: libc::c_int = 0;
const KD_GRAPHICS: libc::c_int = 1;
const FB_BLANK_UNBLANK: libc::c_int = 0;

static CONSOLE_ACTIVE: AtomicBool = AtomicBool::new(false);

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct FbBitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct FbVarScreeninfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: FbBitfield,
    green: FbBitfield,
    blue: FbBitfield,
    transp: FbBitfield,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FbFixScreeninfo {
    id: [u8; 16],
    smem_start: libc::c_ulong,
    smem_len: u32,
    fb_type: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: libc::c_ulong,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

impl Default for FbFixScreeninfo {
    fn default() -> Self {
        // SAFETY: `fb_fix_screeninfo` is a plain C struct of integers; an
        // all-zero value is a valid initial value before `FBIOGET_*`.
        unsafe { std::mem::zeroed() }
    }
}

#[derive(Debug, Clone, Copy)]
struct Channel {
    table: [u32; 256],
}

impl Channel {
    fn new(offset: u32, length: u32) -> Self {
        let max = if length == 0 { 0 } else { (1u32 << length) - 1 };
        let mut table = [0u32; 256];
        for (value, entry) in table.iter_mut().enumerate() {
            *entry = if length == 0 {
                0
            } else {
                ((u32::try_from(value).unwrap_or(0) * max + 127) / 255) << offset
            };
        }
        Self { table }
    }
}

#[derive(Debug, Clone, Copy)]
struct PixelFormat {
    bytes_per_pixel: usize,
    red: Channel,
    green: Channel,
    blue: Channel,
    alpha: Option<Channel>,
    rgba_order: bool,
}

impl PixelFormat {
    fn from_var(var: &FbVarScreeninfo) -> Result<Self, String> {
        if !matches!(var.bits_per_pixel, 16 | 24 | 32) {
            return Err(format!(
                "bits_per_pixel não suportado: {} (esperado 16, 24 ou 32)",
                var.bits_per_pixel
            ));
        }
        for (name, field) in [("R", var.red), ("G", var.green), ("B", var.blue)] {
            if field.length == 0 || field.length > 8 {
                return Err(format!(
                    "canal {name} inválido: offset {} length {}",
                    field.offset, field.length
                ));
            }
            if field.offset.saturating_add(field.length) > var.bits_per_pixel {
                return Err(format!(
                    "canal {name} fora do pixel: offset {} length {} bpp {}",
                    field.offset, field.length, var.bits_per_pixel
                ));
            }
        }
        let alpha = if var.transp.length == 8
            && var.transp.offset.saturating_add(var.transp.length) <= var.bits_per_pixel
        {
            Some(Channel::new(var.transp.offset, var.transp.length))
        } else {
            None
        };
        let rgba_order = var.red.offset == 0 && var.green.offset == 8 && var.blue.offset == 16;
        Ok(Self {
            bytes_per_pixel: usize::try_from(var.bits_per_pixel / 8).unwrap_or(4),
            red: Channel::new(var.red.offset, var.red.length),
            green: Channel::new(var.green.offset, var.green.length),
            blue: Channel::new(var.blue.offset, var.blue.length),
            alpha,
            rgba_order,
        })
    }

    fn pack(&self, red: u8, green: u8, blue: u8) -> u32 {
        let mut value = self.red.table[usize::from(red)]
            | self.green.table[usize::from(green)]
            | self.blue.table[usize::from(blue)];
        if let Some(alpha) = self.alpha {
            value |= alpha.table[255];
        }
        value
    }

    fn write_pixel(&self, output: &mut [u8], red: u8, green: u8, blue: u8) {
        let value = self.pack(red, green, blue);
        for (index, byte) in output.iter_mut().enumerate() {
            *byte = u8::try_from((value >> (8 * index)) & 0xff).unwrap_or(0);
        }
    }
}

pub struct Framebuffer {
    file: File,
    map: *mut u8,
    map_len: usize,
    var: FbVarScreeninfo,
    fix: FbFixScreeninfo,
    format: PixelFormat,
    current_yoffset: u32,
    pan_ok: bool,
}

impl Framebuffer {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_CLOEXEC)
            .open(path)
            .map_err(|error| open_error(path, &error))?;
        let fd = file.as_raw_fd();

        let mut var = FbVarScreeninfo::default();
        // SAFETY: `fd` is a valid framebuffer descriptor and `var` is a
        // correctly sized `fb_var_screeninfo` for the kernel ABI.
        if unsafe { libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var) } < 0 {
            return Err(AppError::FbUnavailable(format!(
                "FBIOGET_VSCREENINFO falhou em {}: {}",
                path.display(),
                std::io::Error::last_os_error()
            )));
        }
        let mut fix = FbFixScreeninfo::default();
        // SAFETY: same descriptor and a correctly sized `fb_fix_screeninfo`.
        if unsafe { libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix) } < 0 {
            return Err(AppError::FbUnavailable(format!(
                "FBIOGET_FSCREENINFO falhou em {}: {}",
                path.display(),
                std::io::Error::last_os_error()
            )));
        }

        validate_geometry(&var, &fix)?;
        let format = PixelFormat::from_var(&var).map_err(AppError::FbUnavailable)?;

        let bytes_per_pixel = u64::try_from(format.bytes_per_pixel).unwrap_or(4);
        let stride = u64::from(fix.line_length);
        let visible_rows = u64::from(var.yoffset) + u64::from(var.yres);
        let required = stride.saturating_mul(visible_rows);
        if u64::from(fix.smem_len) < required {
            return Err(AppError::FbUnavailable(format!(
                "smem_len {} menor que o necessário {} (stride {} × {} linhas)",
                fix.smem_len, required, fix.line_length, visible_rows
            )));
        }
        let expected_row = u64::from(var.xres).saturating_mul(bytes_per_pixel);
        if u64::from(fix.line_length) < expected_row {
            return Err(AppError::FbUnavailable(format!(
                "line_length {} menor que xres × bytes_por_pixel ({})",
                fix.line_length, expected_row
            )));
        }

        let pan_ok = pan_available(&var, &fix);
        let pan_len = stride.saturating_mul(2).saturating_mul(u64::from(var.yres));
        let map_len = if pan_ok {
            pan_len.max(required)
        } else {
            required
        };
        let map_len = usize::try_from(map_len).map_err(|_| {
            AppError::FbUnavailable("mapeamento do framebuffer maior que usize".to_string())
        })?;

        // SAFETY: the descriptor is valid for the lifetime of the mapping and
        // `map_len` covers the visible and (when available) the pan buffer.
        let map = unsafe {
            libc::mmap(
                ptr::null_mut(),
                map_len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if map == libc::MAP_FAILED {
            return Err(AppError::FbUnavailable(format!(
                "mmap de {} bytes falhou: {}",
                map_len,
                std::io::Error::last_os_error()
            )));
        }

        Ok(Self {
            file,
            map: map.cast::<u8>(),
            map_len,
            var,
            fix,
            format,
            current_yoffset: var.yoffset,
            pan_ok,
        })
    }

    pub const fn xres(&self) -> u32 {
        self.var.xres
    }

    pub const fn yres(&self) -> u32 {
        self.var.yres
    }

    pub const fn pan_enabled(&self) -> bool {
        self.pan_ok
    }

    pub fn unblank(&self) {
        // SAFETY: `file` is the framebuffer descriptor; FBIOBLANK takes an int.
        let _ = unsafe { libc::ioctl(self.file.as_raw_fd(), FBIOBLANK, FB_BLANK_UNBLANK) };
    }

    pub fn present(&mut self, pixmap: &Pixmap) -> Result<(), AppError> {
        if pixmap.width() != self.var.xres || pixmap.height() != self.var.yres {
            return Err(AppError::Fb(format!(
                "frame {}×{} não corresponde ao dispositivo {}×{}",
                pixmap.width(),
                pixmap.height(),
                self.var.xres,
                self.var.yres
            )));
        }

        let data = pixmap.data();
        if self.pan_ok {
            let target = if self.current_yoffset == 0 {
                self.var.yres
            } else {
                0
            };
            self.write_frame(data, target);
            if self.try_pan(target) {
                self.current_yoffset = target;
                return Ok(());
            }
            self.pan_ok = false;
        }

        self.write_frame(data, self.current_yoffset);
        Ok(())
    }

    fn try_pan(&self, yoffset: u32) -> bool {
        let mut var = self.var;
        var.yoffset = yoffset;
        // SAFETY: `file` is a valid framebuffer descriptor and `var` is a
        // correctly sized `fb_var_screeninfo`; FBIOPAN_DISPLAY reads it.
        unsafe { libc::ioctl(self.file.as_raw_fd(), FBIOPAN_DISPLAY, &mut var) >= 0 }
    }

    fn write_frame(&mut self, data: &[u8], yoffset: u32) {
        let format = self.format;
        let stride = usize::try_from(self.fix.line_length).unwrap_or(0);
        let width = usize::try_from(self.var.xres).unwrap_or(0);
        let height = usize::try_from(self.var.yres).unwrap_or(0);
        let bytes_per_pixel = format.bytes_per_pixel;
        let source_stride = width.saturating_mul(4);
        let base = usize::try_from(yoffset).unwrap_or(0).saturating_mul(stride);
        let map = self.map_mut();

        for y in 0..height {
            let source = &data[y * source_stride..(y + 1) * source_stride];
            let row = base + y * stride;
            let output = &mut map[row..row + width * bytes_per_pixel];
            if format.rgba_order && bytes_per_pixel == 4 {
                output.copy_from_slice(source);
                continue;
            }
            for (x, pixel) in output.chunks_exact_mut(bytes_per_pixel).enumerate() {
                let offset = x * 4;
                format.write_pixel(
                    pixel,
                    source[offset],
                    source[offset + 1],
                    source[offset + 2],
                );
            }
        }
    }

    fn map_mut(&mut self) -> &mut [u8] {
        // SAFETY: `map` points to `map_len` bytes mapped in `open`; `&mut self`
        // guarantees exclusive access for the duration of the slice.
        unsafe { std::slice::from_raw_parts_mut(self.map, self.map_len) }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        // SAFETY: `map` and `map_len` come from the successful `mmap` in
        // `open` and are unmapped exactly once.
        unsafe {
            libc::munmap(self.map.cast(), self.map_len);
        }
    }
}

pub struct ConsoleGuard {
    tty: File,
    active: bool,
}

impl ConsoleGuard {
    pub fn enter() -> Result<Self, AppError> {
        let tty = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_CLOEXEC)
            .open("/dev/tty")
            .map_err(|error| {
                AppError::FbUnavailable(format!("{} ({error})", vt_error_at(&current_tty())))
            })?;
        let fd = tty.as_raw_fd();
        let name = tty_name(fd);

        let mut mode: libc::c_int = 0;
        // SAFETY: `tty` is a valid descriptor and KDGETMODE writes an int.
        if unsafe { libc::ioctl(fd, KDGETMODE, &mut mode) } < 0 {
            return Err(AppError::FbUnavailable(vt_error_at(&name)));
        }
        // SAFETY: same descriptor; KDSETMODE takes an int argument.
        if unsafe { libc::ioctl(fd, KDSETMODE, KD_GRAPHICS) } < 0 {
            return Err(AppError::FbUnavailable(format!(
                "{} (KDSETMODE(KD_GRAPHICS): {})",
                vt_error_at(&name),
                std::io::Error::last_os_error()
            )));
        }
        CONSOLE_ACTIVE.store(true, Ordering::SeqCst);
        if let Err(error) = enable_raw_mode() {
            CONSOLE_ACTIVE.store(false, Ordering::SeqCst);
            restore_text_mode(fd);
            return Err(AppError::Terminal(error));
        }

        let mut guard = Self { tty, active: true };
        let _ = guard.tty.write_all(b"\x1b[?25l\x1b[2J\x1b[H");
        Ok(guard)
    }

    pub fn restore(&mut self) {
        if self.active {
            emergency_restore();
            self.active = false;
        }
    }
}

impl Drop for ConsoleGuard {
    fn drop(&mut self) {
        self.restore();
    }
}

pub fn emergency_restore() {
    if !CONSOLE_ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }
    let _ = disable_raw_mode();
    if let Ok(mut tty) = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_CLOEXEC)
        .open("/dev/tty")
    {
        restore_text_mode(tty.as_raw_fd());
        let _ = tty.write_all(b"\x1b[0m\x1b[?25h\x1b[2J\x1b[H");
    }
}

fn restore_text_mode(fd: RawFd) {
    // SAFETY: `fd` is a valid terminal descriptor (or was at call time);
    // KDSETMODE takes an int and is best-effort during restoration.
    let _ = unsafe { libc::ioctl(fd, KDSETMODE, KD_TEXT) };
}

fn validate_geometry(var: &FbVarScreeninfo, fix: &FbFixScreeninfo) -> Result<(), AppError> {
    if var.xres == 0 || var.yres == 0 {
        return Err(AppError::FbUnavailable(format!(
            "resolução inválida: {}×{}",
            var.xres, var.yres
        )));
    }
    if var.yres_virtual < var.yres {
        return Err(AppError::FbUnavailable(format!(
            "yres_virtual {} menor que yres {}",
            var.yres_virtual, var.yres
        )));
    }
    if var.yoffset > var.yres_virtual - var.yres {
        return Err(AppError::FbUnavailable(format!(
            "yoffset {} fora da área virtual (yres_virtual {})",
            var.yoffset, var.yres_virtual
        )));
    }
    if fix.line_length == 0 || fix.smem_len == 0 {
        return Err(AppError::FbUnavailable(
            "line_length ou smem_len zerado".to_string(),
        ));
    }
    Ok(())
}

fn pan_available(var: &FbVarScreeninfo, fix: &FbFixScreeninfo) -> bool {
    if fix.ypanstep == 0 || !var.yres.is_multiple_of(u32::from(fix.ypanstep)) {
        return false;
    }
    if var.yres_virtual < var.yres.saturating_mul(2) {
        return false;
    }
    let required = u64::from(fix.line_length)
        .saturating_mul(2)
        .saturating_mul(u64::from(var.yres));
    u64::from(fix.smem_len) >= required
}

fn open_error(path: &Path, error: &std::io::Error) -> AppError {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        return AppError::FbUnavailable(i18n::fill(
            Key::ErrorFbPermission.text(Language::Pt),
            &[("path", &path.display().to_string())],
        ));
    }
    AppError::FbUnavailable(format!(
        "não foi possível abrir {}: {error}",
        path.display()
    ))
}

fn current_tty() -> String {
    std::fs::read_link("/proc/self/fd/0")
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "desconhecido".to_string())
}

fn vt_error_at(tty: &str) -> String {
    i18n::fill(Key::ErrorFbVt.text(Language::Pt), &[("tty", tty)])
}

fn tty_name(fd: RawFd) -> String {
    // SAFETY: `ttyname` returns either null or a pointer to a static
    // NUL-terminated string owned by libc; it is not retained.
    unsafe {
        let pointer = libc::ttyname(fd);
        if pointer.is_null() {
            current_tty()
        } else {
            CStr::from_ptr(pointer).to_string_lossy().into_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    fn var_with_channels(
        bits: u32,
        red: (u32, u32),
        green: (u32, u32),
        blue: (u32, u32),
    ) -> FbVarScreeninfo {
        let mut var = FbVarScreeninfo {
            bits_per_pixel: bits,
            ..FbVarScreeninfo::default()
        };
        var.red = FbBitfield {
            offset: red.0,
            length: red.1,
            msb_right: 0,
        };
        var.green = FbBitfield {
            offset: green.0,
            length: green.1,
            msb_right: 0,
        };
        var.blue = FbBitfield {
            offset: blue.0,
            length: blue.1,
            msb_right: 0,
        };
        var
    }

    #[test]
    fn kernel_structs_match_the_x86_64_abi() {
        assert_eq!(size_of::<FbVarScreeninfo>(), 160);
        assert_eq!(size_of::<FbFixScreeninfo>(), 80);
    }

    #[test]
    fn builds_rgba32_format() {
        let var = var_with_channels(32, (0, 8), (8, 8), (16, 8));
        let format = PixelFormat::from_var(&var).unwrap();
        assert!(format.rgba_order);
        assert_eq!(format.bytes_per_pixel, 4);
        assert_eq!(format.pack(0x12, 0x34, 0x56), 0x0056_3412);
        let mut pixel = [0u8; 4];
        format.write_pixel(&mut pixel, 1, 2, 3);
        assert_eq!(pixel, [1, 2, 3, 0]);
    }

    #[test]
    fn builds_bgrx32_and_rgb565_formats() {
        let bgr = PixelFormat::from_var(&var_with_channels(32, (16, 8), (8, 8), (0, 8))).unwrap();
        assert!(!bgr.rgba_order);
        assert_eq!(bgr.pack(0x12, 0x34, 0x56), 0x0012_3456);

        let rgb565 =
            PixelFormat::from_var(&var_with_channels(16, (11, 5), (5, 6), (0, 5))).unwrap();
        assert_eq!(rgb565.bytes_per_pixel, 2);
        assert_eq!(rgb565.pack(0xff, 0, 0), 0xf800);
        assert_eq!(rgb565.pack(0, 0xff, 0), 0x07e0);
        assert_eq!(rgb565.pack(0, 0, 0xff), 0x001f);
    }

    #[test]
    fn builds_rgb24_and_writes_three_bytes() {
        let var = var_with_channels(24, (0, 8), (8, 8), (16, 8));
        let format = PixelFormat::from_var(&var).unwrap();
        assert_eq!(format.bytes_per_pixel, 3);
        assert!(format.rgba_order);
        assert_eq!(format.pack(0x12, 0x34, 0x56), 0x0056_3412);
        let mut pixel = [0u8; 3];
        format.write_pixel(&mut pixel, 0x12, 0x34, 0x56);
        assert_eq!(pixel, [0x12, 0x34, 0x56]);

        let bgr = PixelFormat::from_var(&var_with_channels(24, (16, 8), (8, 8), (0, 8))).unwrap();
        assert!(!bgr.rgba_order);
        assert_eq!(bgr.pack(0x12, 0x34, 0x56), 0x0012_3456);
    }

    #[test]
    fn writes_rgb565_as_little_endian_bytes() {
        let format =
            PixelFormat::from_var(&var_with_channels(16, (11, 5), (5, 6), (0, 5))).unwrap();
        let mut pixel = [0u8; 2];
        format.write_pixel(&mut pixel, 0xff, 0, 0);
        assert_eq!(pixel, [0x00, 0xf8]);
    }

    #[test]
    fn rejects_unsupported_pixel_formats() {
        assert!(PixelFormat::from_var(&var_with_channels(8, (0, 3), (3, 3), (6, 2))).is_err());
        assert!(PixelFormat::from_var(&var_with_channels(32, (0, 0), (8, 8), (16, 8))).is_err());
        assert!(PixelFormat::from_var(&var_with_channels(16, (12, 5), (5, 6), (0, 5))).is_err());
    }

    #[test]
    fn pan_requires_step_double_buffer_and_room() {
        let var = FbVarScreeninfo {
            yres: 1080,
            yres_virtual: 2160,
            ..FbVarScreeninfo::default()
        };
        let fix = FbFixScreeninfo {
            line_length: 7680,
            smem_len: 7680 * 2160,
            ypanstep: 1,
            ..FbFixScreeninfo::default()
        };
        assert!(pan_available(&var, &fix));

        let no_step = FbFixScreeninfo { ypanstep: 0, ..fix };
        assert!(!pan_available(&var, &no_step));

        let single = FbVarScreeninfo {
            yres_virtual: 1080,
            ..var
        };
        assert!(!pan_available(&single, &fix));

        let small = FbFixScreeninfo {
            smem_len: 7680 * 1080,
            ..fix
        };
        assert!(!pan_available(&var, &small));
    }

    #[test]
    fn geometry_validation_rejects_bad_offsets() {
        let var = FbVarScreeninfo {
            xres: 1920,
            yres: 1080,
            yres_virtual: 1080,
            yoffset: 1,
            ..FbVarScreeninfo::default()
        };
        let fix = FbFixScreeninfo {
            line_length: 7680,
            smem_len: 7680 * 1080,
            ..FbFixScreeninfo::default()
        };
        assert!(validate_geometry(&var, &fix).is_err());
    }
}
