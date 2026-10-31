#!/usr/bin/env bash
set -euo pipefail

APP="piano-duas-oitavas"
DISPLAY_NAME="Piano Duas Oitavas"
VERSION="0.1.0"
PKG="${APP}-${VERSION}-x86_64-unknown-linux-gnu"
PREFIX="${PREFIX:-/usr/local}"
SKIP_GROUP="${SKIP_GROUP:-0}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SOURCE=""
SOURCE_DIR=""
TARBALL=""
TMP_DIR=""
NEED_RELOGIN=0

usage() {
    cat <<EOF
Uso: $(basename "$0") [opções] [pacote.tar.gz | diretório]

Instala $DISPLAY_NAME e libera o acesso do usuário atual ao framebuffer.

Opções:
  --prefix DIR    destino da instalação (padrão: /usr/local)
  --skip-group    não altera o grupo video (útil para testes/CI)
  -h, --help      mostra esta ajuda

Sem argumento, procura ${PKG}.tar.gz no diretório atual, no diretório do
script ou usa o próprio diretório já extraído.
EOF
}

cleanup() {
    [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ] && rm -rf "$TMP_DIR"
}
trap cleanup EXIT

fail() {
    echo "erro: $*" >&2
    exit 1
}

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)
            [ $# -ge 2 ] || fail "--prefix precisa de um diretório"
            PREFIX="$2"
            shift 2
            ;;
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        --skip-group)
            SKIP_GROUP=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            usage >&2
            fail "opção desconhecida: $1"
            ;;
        *)
            [ -z "$SOURCE" ] || fail "informe apenas um pacote ou diretório"
            SOURCE="$1"
            shift
            ;;
    esac
done

[ "$(uname -s)" = "Linux" ] || fail "este instalador é exclusivo para Linux"
[ -n "$PREFIX" ] || fail "o prefixo de instalação está vazio"

RUN_USER="${SUDO_USER:-$(logname 2>/dev/null || id -un)}"
[ -n "$RUN_USER" ] || fail "não foi possível identificar o usuário atual"
id "$RUN_USER" >/dev/null 2>&1 || fail "usuário inexistente: $RUN_USER"

find_source() {
    if [ -n "$SOURCE" ]; then
        [ -e "$SOURCE" ] || fail "caminho não encontrado: $SOURCE"
        printf '%s\n' "$SOURCE"
        return 0
    fi
    if [ -x "$SCRIPT_DIR/$APP" ]; then
        printf '%s\n' "$SCRIPT_DIR"
        return 0
    fi
    for candidate in \
        "$SCRIPT_DIR/$PKG.tar.gz" \
        "$PWD/$PKG.tar.gz" \
        "$SCRIPT_DIR/dist/$PKG.tar.gz" \
        "$PWD/dist/$PKG.tar.gz" \
        "$SCRIPT_DIR"/${APP}-*-x86_64-unknown-linux-gnu.tar.gz \
        "$PWD"/${APP}-*-x86_64-unknown-linux-gnu.tar.gz
    do
        [ -f "$candidate" ] && { printf '%s\n' "$candidate"; return 0; }
    done
    return 1
}

RESOLVED="$(find_source)" || fail "pacote não encontrado; informe o caminho do .tar.gz"

case "$RESOLVED" in
    *.tar.gz)
        TARBALL="$RESOLVED"
        if [ -f "$TARBALL.sha256" ]; then
            echo "Verificando SHA-256..."
            ( cd -- "$(dirname -- "$TARBALL")" && sha256sum -c "$(basename -- "$TARBALL").sha256" )
        fi
        command -v tar >/dev/null 2>&1 || fail "tar não encontrado"
        TMP_DIR="$(mktemp -d)"
        tar -xzf "$TARBALL" -C "$TMP_DIR"
        SOURCE_DIR="$(find "$TMP_DIR" -mindepth 1 -maxdepth 1 -type d -print -quit)"
        if [ -z "$SOURCE_DIR" ] || [ ! -x "$SOURCE_DIR/$APP" ]; then
            [ -x "$TMP_DIR/$APP" ] && SOURCE_DIR="$TMP_DIR" || fail "o pacote não contém o binário $APP"
        fi
        ;;
    *)
        SOURCE_DIR="$RESOLVED"
        ;;
esac

[ -d "$SOURCE_DIR" ] || fail "diretório de origem inválido: $SOURCE_DIR"
[ -x "$SOURCE_DIR/$APP" ] || fail "binário não encontrado ou sem permissão: $SOURCE_DIR/$APP"

INSTALLER=()
if [ "$(id -u)" -ne 0 ]; then
    if [ -w "$PREFIX" ] || { [ ! -e "$PREFIX" ] && [ -w "$(dirname -- "$PREFIX")" ]; }; then
        INSTALLER=()
    else
        command -v sudo >/dev/null 2>&1 || fail "sudo é necessário para instalar em $PREFIX"
        INSTALLER=(sudo)
    fi
fi

run_install() { "${INSTALLER[@]}" "$@"; }

as_root() {
    if [ "$(id -u)" -eq 0 ]; then "$@"; else
        command -v sudo >/dev/null 2>&1 || fail "sudo é necessário para configurar o framebuffer"
        sudo "$@"
    fi
}

echo "Instalando $DISPLAY_NAME $VERSION em $PREFIX..."
run_install install -d "$PREFIX/bin" "$PREFIX/share/$APP"
run_install install -m755 "$SOURCE_DIR/$APP" "$PREFIX/bin/$APP"

for doc in README.md LICENSE LICENSE-ART THIRD_PARTY_NOTICES.md RELEASE_NOTES.md CREDITS.md; do
    [ -f "$SOURCE_DIR/$doc" ] && run_install install -m644 "$SOURCE_DIR/$doc" "$PREFIX/share/$APP/$doc"
done
[ -f "$SOURCE_DIR/testes_debug.sh" ] && run_install install -m755 "$SOURCE_DIR/testes_debug.sh" "$PREFIX/share/$APP/testes_debug.sh"

if [ -d "$SOURCE_DIR/licenses" ]; then
    run_install install -d "$PREFIX/share/$APP/licenses"
    for license in "$SOURCE_DIR"/licenses/*; do
        [ -f "$license" ] && run_install install -m644 "$license" "$PREFIX/share/$APP/licenses/"
    done
fi

if [ "$SKIP_GROUP" -eq 0 ]; then
    if ! getent group video >/dev/null 2>&1; then
        echo "Criando o grupo video..."
        as_root groupadd --system video
    fi
    if id -nG "$RUN_USER" 2>/dev/null | tr ' ' '\n' | grep -qx video; then
        echo "O usuário $RUN_USER já pertence ao grupo video."
    else
        echo "Adicionando $RUN_USER ao grupo video (pode pedir a senha do sudo)..."
        as_root usermod -aG video "$RUN_USER"
        NEED_RELOGIN=1
    fi
    [ -e /dev/fb0 ] && ls -l /dev/fb0 || echo "aviso: /dev/fb0 não existe; o backend fb exigirá um VT com framebuffer."
else
    echo "Grupo video não alterado (--skip-group)."
fi

cat <<EOF

Instalação concluída.

  Binário: $PREFIX/bin/$APP
  Documentos: $PREFIX/share/$APP

Console (VT local, fora de X/Wayland):
  $PREFIX/bin/$APP --backend fb

Terminal (TUI):
  $PREFIX/bin/$APP
EOF

if [ "$NEED_RELOGIN" -eq 1 ]; then
    echo "O acesso ao framebuffer só vale após sair e entrar novamente na sessão."
    echo "Para aplicar agora neste shell, use: newgrp video"
fi

if [ "$PREFIX" = "/usr/local" ] && ! printf '%s' "$PATH" | tr ':' '\n' | grep -qx "/usr/local/bin"; then
    echo "aviso: /usr/local/bin não está no PATH deste shell."
fi
