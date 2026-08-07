#!/bin/bash
set -e

echo "Compilando o projeto..."
./build.sh

echo "Instalando o SwayManager e SwayManagerGUI..."
rm -rf ~/.config/sway/bin/sway-manager
rm -f ~/.config/sway/bin/SwayManager
rm -f ~/.config/sway/bin/sway-manager
rm -f ~/.config/sway/bin/SwayManagerGUI
rm -f ~/.config/sway/bin/sway-manager-gui
rm -f ~/.config/sway/bin/SwayManagerDaemon

mkdir -p ~/.config/sway/bin/sway-manager
cp -rf ./out/main.dist/* ~/.config/sway/bin/sway-manager/

ln -sf $HOME/.config/sway/bin/sway-manager/SwayManager $HOME/.config/sway/bin/SwayManager
ln -sf $HOME/.config/sway/bin/sway-manager/sway-manager $HOME/.config/sway/bin/sway-manager
ln -sf $HOME/.config/sway/bin/sway-manager/SwayManagerGUI $HOME/.config/sway/bin/SwayManagerGUI
ln -sf $HOME/.config/sway/bin/sway-manager/sway-manager-gui $HOME/.config/sway/bin/sway-manager-gui

INSTALL_UDEV=true
INSTALL_LSWT=false

for arg in "$@"; do
    case "$arg" in
        --no-udev|--skip-udev)
            INSTALL_UDEV=false
            ;;
        --with-lswt|--install-lswt)
            INSTALL_LSWT=true
            ;;
    esac
done

PYTHON_BIN="python3"
if [ -x "venv/bin/python3" ]; then
    PYTHON_BIN="venv/bin/python3"
fi

echo "Configurando XDG Desktop Portal..."
PYTHONPATH=src $PYTHON_BIN - <<'PY'
from portal.config_installer import PortalConfigInstaller
import sys

try:
    installer = PortalConfigInstaller()
    log = installer.install()
    for line in log:
        print(f"  {line}")
except Exception as exc:
    print(f"Aviso: falha ao configurar portal: {exc}", file=sys.stderr)
PY

if [ "$INSTALL_UDEV" = true ]; then
    echo "Instalando regras udev para conservação de bateria (requer sudo)..."
    if [ -f ./udev/99-battery-conservation.rules ]; then
        sudo cp ./udev/99-battery-conservation.rules /etc/udev/rules.d/
        sudo udevadm control --reload-rules
        sudo udevadm trigger
        echo "Regras udev instaladas com sucesso!"
    fi
fi

echo "Verificando utilitário lswt (para compartilhamento de janelas)..."
if command -v lswt &>/dev/null; then
    echo "  lswt já está instalado: $(command -v lswt)"
elif [ "$INSTALL_LSWT" = true ]; then
    echo "Instalando lswt..."
    TMP_LSWT=$(mktemp -d)
    echo "Clonando e compilando lswt (v2.0.0) em $TMP_LSWT..."
    git clone https://git.sr.ht/~leon_plickat/lswt "$TMP_LSWT"
    (
        cd "$TMP_LSWT"
        git checkout v2.0.0
        make
        sudo make install
    )
    rm -rf "$TMP_LSWT"
    echo "lswt instalado com sucesso!"
else
    echo "  Aviso: lswt não encontrado. O compartilhamento de tela funcionará para monitores."
    echo "  Para instalar o lswt automaticamente, execute: ./install.sh --install-lswt"
fi

XDPW_BIN=/usr/libexec/xdg-desktop-portal-wlr
NEEDS_XDPW_BUILD=false

if [ ! -x "$XDPW_BIN" ]; then
    NEEDS_XDPW_BUILD=true
elif ! strings "$XDPW_BIN" 2>/dev/null | grep -q "^Window: %s"; then
    NEEDS_XDPW_BUILD=true
elif ! strings "$XDPW_BIN" 2>/dev/null | grep -q "cached_label"; then
    echo "  xdg-desktop-portal-wlr não possui o patch de label caching; reconstruindo..."
    NEEDS_XDPW_BUILD=true
fi

if [ "$NEEDS_XDPW_BUILD" = false ]; then
    echo "  xdg-desktop-portal-wlr já suporta captura de janela e tem o patch aplicado"
else
    TMP_XDPW=$(mktemp -d)
    echo "Clonando xdg-desktop-portal-wlr v0.8.4 em $TMP_XDPW..."
    git clone --branch v0.8.4 --depth 1 https://github.com/emersion/xdg-desktop-portal-wlr.git "$TMP_XDPW" 2>&1 | tail -1

    echo "Aplicando patch: cache de labels no chooser.c (corrige bug de tripla invocação)..."
    cat > "$TMP_XDPW/src/screencast/chooser.c" << 'PATCH_EOF'
#include <sys/wait.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>

#include "logger.h"
#include "screencast.h"
#include "string_util.h"
#include "wlr_screencast.h"
#include "xdpw.h"
#include "screencast_common.h"
#include <wayland-client.h>

struct cached_label {
	struct wl_list link;
	char *label;
	enum source_types type;
	void *source;
};

static struct xdpw_wlr_output *xdpw_wlr_output_first(struct wl_list *output_list) {
	struct xdpw_wlr_output *output, *tmp;
	wl_list_for_each_safe(output, tmp, output_list, link) {
		return output;
	}
	return NULL;
}

static pid_t spawn_chooser(const char *cmd, FILE **chooser_in_ptr, FILE **chooser_out_ptr) {
	int chooser_in[2];
	int chooser_out[2];

	if (pipe(chooser_in) == -1) {
		perror("pipe chooser_in");
		logprint(ERROR, "Failed to open pipe chooser_in");
		return -1;
	}
	if (pipe(chooser_out) == -1) {
		perror("pipe chooser_out");
		logprint(ERROR, "Failed to open pipe chooser_out");
		goto error_chooser_in;
	}

	logprint(TRACE,
		"exec chooser called: cmd %s, pipe chooser_in (%d,%d), pipe chooser_out (%d,%d)",
		cmd, chooser_in[0], chooser_in[1], chooser_out[0], chooser_out[1]);

	pid_t pid = fork();
	if (pid < 0) {
		perror("fork");
		goto error_chooser_out;
	} else if (pid == 0) {
		close(chooser_in[1]);
		close(chooser_out[0]);

		dup2(chooser_in[0], STDIN_FILENO);
		dup2(chooser_out[1], STDOUT_FILENO);
		close(chooser_in[0]);
		close(chooser_out[1]);

		execl("/bin/sh", "/bin/sh", "-c", cmd, NULL);

		perror("execl");
		_exit(127);
	}

	close(chooser_in[0]);
	close(chooser_out[1]);

	FILE *chooser_in_f = fdopen(chooser_in[1], "w");
	if (chooser_in_f == NULL) {
		close(chooser_in[1]);
		close(chooser_out[0]);
		return -1;
	}
	FILE *chooser_out_f = fdopen(chooser_out[0], "r");
	if (chooser_out_f == NULL) {
		fclose(chooser_in_f);
		close(chooser_out[0]);
		return -1;
	}

	*chooser_in_ptr = chooser_in_f;
	*chooser_out_ptr = chooser_out_f;

	return pid;

error_chooser_out:
	close(chooser_out[0]);
	close(chooser_out[1]);
error_chooser_in:
	close(chooser_in[0]);
	close(chooser_in[1]);
	return -1;
}

static bool wait_chooser(pid_t pid) {
	int status;
	if (waitpid(pid, &status, 0) != -1 && WIFEXITED(status)) {
		return WEXITSTATUS(status) != 127;
	}
	return false;
}

static char *read_chooser_out(FILE *f) {
	char *name = NULL;
	size_t name_size = 0;
	ssize_t nread = getline(&name, &name_size, f);
	if (nread < 0) {
		if (!feof(f)) {
			perror("getline failed");
		}
		return NULL;
	}

	char *p = strchr(name, '\n');
	if (p != NULL) {
		*p = '\0';
	}

	return name;
}

static char *get_output_label(struct xdpw_wlr_output *output, enum xdpw_chooser_types chooser_type) {
	if (chooser_type == XDPW_CHOOSER_DMENU) {
		return format_str("Monitor: %s %s", output->name, output->description);
	} else {
		return format_str("Monitor: %s", output->name);
	}
}

static char *get_toplevel_label(struct xdpw_toplevel *toplevel, enum xdpw_chooser_types chooser_type) {
	if (chooser_type == XDPW_CHOOSER_DMENU) {
		return format_str("Window: %s (%s)", toplevel->title, toplevel->identifier);
	} else {
		return format_str("Window: %s", toplevel->identifier);
	}
}

static void cached_labels_free(struct wl_list *cache) {
	struct cached_label *cl, *tmp;
	wl_list_for_each_safe(cl, tmp, cache, link) {
		wl_list_remove(&cl->link);
		free(cl->label);
		free(cl);
	}
}

static bool wlr_chooser(const struct xdpw_chooser *chooser,
		struct xdpw_screencast_context *ctx, struct xdpw_screencast_target *target,
		uint32_t type_mask) {
	logprint(DEBUG, "wlroots: chooser called");

	FILE *chooser_in = NULL, *chooser_out = NULL;
	pid_t pid = spawn_chooser(chooser->cmd, &chooser_in, &chooser_out);
	if (pid < 0) {
		logprint(ERROR, "Failed to fork chooser");
		return false;
	}

	struct wl_list cache;
	wl_list_init(&cache);

	switch (chooser->type) {
	case XDPW_CHOOSER_DMENU:
		if (type_mask & MONITOR) {
			struct xdpw_wlr_output *out;
			wl_list_for_each(out, &ctx->output_list, link) {
				char *label = get_output_label(out, chooser->type);
				fprintf(chooser_in, "%s\n", label);
				struct cached_label *cl = calloc(1, sizeof(*cl));
				cl->label = label;
				cl->type = MONITOR;
				cl->source = out;
				wl_list_insert(cache.prev, &cl->link);
			}
		}
		if (type_mask & WINDOW) {
			struct xdpw_toplevel *toplevel;
			wl_list_for_each(toplevel, &ctx->toplevels, link) {
				char *label = get_toplevel_label(toplevel, chooser->type);
				fprintf(chooser_in, "%s\n", label);
				struct cached_label *cl = calloc(1, sizeof(*cl));
				cl->label = label;
				cl->type = WINDOW;
				cl->source = toplevel;
				wl_list_insert(cache.prev, &cl->link);
			}
		}
		fclose(chooser_in);
		break;
	default:
		fclose(chooser_in);
	}

	if (!wait_chooser(pid)) {
		fclose(chooser_out);
		cached_labels_free(&cache);
		return false;
	}

	char *selected_label = read_chooser_out(chooser_out);
	fclose(chooser_out);
	if (selected_label == NULL) {
		cached_labels_free(&cache);
		return true;
	}

	logprint(TRACE, "wlroots: chooser %s selects %s", chooser->cmd, selected_label);

	bool found = false;
	struct cached_label *cl;
	wl_list_for_each(cl, &cache, link) {
		found = strcmp(selected_label, cl->label) == 0;
		if (!found && chooser->type == XDPW_CHOOSER_SIMPLE && cl->type == MONITOR) {
			struct xdpw_wlr_output *out = cl->source;
			found = strcmp(selected_label, out->name) == 0;
		}
		if (found) {
			if (cl->type == MONITOR) {
				target->type = MONITOR;
				target->output = cl->source;
			} else {
				target->type = WINDOW;
				target->toplevel = cl->source;
			}
			break;
		}
	}

	if (!found) {
		logprint(ERROR, "wlroots: chooser %s selected unknown target: %s", chooser->cmd, selected_label);
	}

	free(selected_label);
	cached_labels_free(&cache);

	return true;
}

static bool wlr_chooser_default(struct xdpw_screencast_context *ctx, struct xdpw_screencast_target *target, uint32_t type_mask) {
	logprint(DEBUG, "wlroots: chooser called");

	const struct xdpw_chooser default_chooser[] = {
		{XDPW_CHOOSER_SIMPLE, "slurp -f 'Monitor: %o' -or"},
		{XDPW_CHOOSER_DMENU, "wmenu -p 'Select a source to share:' -l 10"},
		{XDPW_CHOOSER_DMENU, "wofi -d -n --prompt='Select a source to share:'"},
		{XDPW_CHOOSER_DMENU, "rofi -dmenu -p 'Select a source to share:'"},
		{XDPW_CHOOSER_DMENU, "bemenu --prompt='Select a source to share:'"},
		{XDPW_CHOOSER_DMENU, "mew -l 10 -p 'Select a source to share:'"},
		{XDPW_CHOOSER_DMENU, "fuzzel -d -l 10 -p 'Select a source to share:'"},
	};

	size_t N = sizeof(default_chooser)/sizeof(default_chooser[0]);
	bool ret;
	for (size_t i = 0; i<N; i++) {
		const struct xdpw_chooser *chooser = &default_chooser[i];
		if (chooser->type == XDPW_CHOOSER_SIMPLE && type_mask != MONITOR) {
			continue;
		}
		ret = wlr_chooser(chooser, ctx, target, type_mask);
		if (!ret) {
			logprint(DEBUG, "wlroots: chooser %s failed. Trying next one.",
					default_chooser[i].cmd);
			continue;
		}
		return target->output || target->toplevel;
	}
	return false;
}

bool xdpw_wlr_target_chooser(struct xdpw_screencast_context *ctx, struct xdpw_screencast_target *target, uint32_t type_mask) {
	switch (ctx->state->config->screencast_conf.chooser_type) {
	case XDPW_CHOOSER_DEFAULT:
		return wlr_chooser_default(ctx, target, type_mask);
	case XDPW_CHOOSER_NONE:
		target->type = MONITOR;
		if (ctx->state->config->screencast_conf.output_name) {
			target->output = xdpw_wlr_output_find_by_name(&ctx->output_list, ctx->state->config->screencast_conf.output_name);
		} else {
			target->output = xdpw_wlr_output_first(&ctx->output_list);
		}
		return target->output != NULL;
	case XDPW_CHOOSER_DMENU:
	case XDPW_CHOOSER_SIMPLE:;
		if (!ctx->state->config->screencast_conf.chooser_cmd) {
			logprint(ERROR, "wlroots: no chooser given");
			goto end;
		}
		struct xdpw_chooser chooser = {
			ctx->state->config->screencast_conf.chooser_type,
			ctx->state->config->screencast_conf.chooser_cmd
		};
		logprint(DEBUG, "wlroots: chooser %s (%d)", chooser.cmd, chooser.type);
		bool ret = wlr_chooser(&chooser, ctx, target, type_mask);
		if (!ret) {
			logprint(ERROR, "wlroots: chooser %s failed", chooser.cmd);
			goto end;
		}
		return target->output || target->toplevel;
	}
end:
	return false;
}
PATCH_EOF

    echo "Compilando e instalando xdg-desktop-portal-wlr v0.8.4 com patch..."
    meson setup "$TMP_XDPW-build" "$TMP_XDPW" --prefix=/usr -Dsd-bus-provider=libsystemd 2>&1 | tail -3
    ninja -C "$TMP_XDPW-build" 2>&1 | tail -3
    sudo ninja -C "$TMP_XDPW-build" install 2>&1 | tail -5
    rm -rf "$TMP_XDPW" "$TMP_XDPW-build"
    echo "  xdg-desktop-portal-wlr v0.8.4 (com patch) instalado com sucesso!"
fi

echo "Instalação concluída."