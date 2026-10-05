"""Test the packaged settings UI on an isolated X11 display and configuration.

Run with xvfb-run -a dbus-run-session -- python3 tests/linux_appimage_smoke.py APPIMAGE.
Requires python3-pyatspi, python3-gi, at-spi2-core, xdotool, xvfb and xauth.
"""

from collections import deque
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time


def walk(root):
    queue = deque([root])
    for _ in range(400):
        if not queue:
            return
        node = queue.popleft()
        yield node
        queue.extend(node)


def find(root, predicate):
    return next((node for node in walk(root) if predicate(node)), None)


def wait_for(check, application, description):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        if application.poll() is not None:
            raise RuntimeError(f"Application exited: {application.returncode}")
        result = check()
        if result is not None:
            return result
        time.sleep(0.1)
    raise RuntimeError(f"Timed out waiting for {description}")


def command(*args):
    return subprocess.run(
        args, check=True, capture_output=True, text=True, timeout=5
    ).stdout


def stop(process):
    if process is None or process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=3)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def main(appimage):
    # Never send mouse events to the user's real desktop if run without xvfb-run.
    if not Path(os.environ.get("XAUTHORITY", "")).parent.name.startswith("xvfb-run."):
        raise RuntimeError("Run this UI test under xvfb-run with its default Xauthority")
    registry_candidates = (
        shutil.which("at-spi2-registryd"),
        "/usr/libexec/at-spi2-registryd",
        "/usr/lib/at-spi2-core/at-spi2-registryd",
    )
    registry_binary = next(
        (path for path in registry_candidates if path and Path(path).is_file()), None
    )
    if registry_binary is None:
        raise RuntimeError("at-spi2-registryd is required")

    with tempfile.TemporaryDirectory(prefix="transhelper-ui-") as directory:
        root = Path(directory)
        env = dict(os.environ)
        # Never let an inherited real Wayland socket bypass the isolated Xvfb.
        env.pop("WAYLAND_DISPLAY", None)
        env.pop("WAYLAND_SOCKET", None)
        env["XDG_SESSION_TYPE"] = "x11"
        env["GDK_BACKEND"] = "x11"
        for key, subdir in (
            ("XDG_CONFIG_HOME", "config"),
            ("XDG_DATA_HOME", "data"),
            ("XDG_CACHE_HOME", "cache"),
        ):
            (root / subdir).mkdir()
            env[key] = str(root / subdir)
        application = registry = bus = None
        with (root / "app.log").open("w") as log:
            try:
                # Exercise the real AppRun GTK hook and bundled libraries without FUSE.
                subprocess.run(
                    [str(appimage), "--appimage-extract"], cwd=root, stdout=log,
                    stderr=subprocess.STDOUT, check=True, timeout=30,
                )
                # A private accessibility bus prevents inspecting unrelated windows.
                bus = subprocess.Popen(
                    [
                        "dbus-daemon", "--session", "--nofork", "--print-address",
                        f"--address=unix:path={root / 'a11y-bus'}",
                    ],
                    stdout=subprocess.PIPE, stderr=log, text=True,
                )
                env["AT_SPI_BUS_ADDRESS"] = bus.stdout.readline().strip()
                if not env["AT_SPI_BUS_ADDRESS"]:
                    raise RuntimeError("Accessibility bus did not start")
                os.environ["AT_SPI_BUS_ADDRESS"] = env["AT_SPI_BUS_ADDRESS"]
                registry = subprocess.Popen(
                    [registry_binary], env=env, stdout=log, stderr=subprocess.STDOUT
                )

                import gi
                gi.require_version("Atspi", "2.0")
                from gi.repository import Atspi
                import pyatspi
                Atspi.set_timeout(3000, 3000)

                application = subprocess.Popen(
                    [str(root / "squashfs-root/AppRun")], env=env,
                    stdout=log, stderr=subprocess.STDOUT,
                )
                desktop = pyatspi.Registry.getDesktop(0)
                app = wait_for(
                    lambda: find(desktop, lambda n: n.getRoleName() == "application"
                                 and n.get_process_id() == application.pid),
                    application, "application",
                )

                def button(name):
                    return find(
                        app, lambda n: n.getRoleName() in ("button", "push button")
                        and n.name == name,
                    )

                settings = wait_for(lambda: button("设置"), application, "settings button")
                window_id = command(
                    "xdotool", "search", "--onlyvisible", "--pid", str(application.pid)
                ).split()[-1]

                def click(node):
                    node.queryComponent().scrollTo(pyatspi.SCROLL_ANYWHERE)
                    time.sleep(0.15)
                    bounds = node.queryComponent().getExtents(pyatspi.DESKTOP_COORDS)
                    assert bounds.width > 0 and bounds.height > 0
                    command("xdotool", "windowfocus", "--sync", window_id)
                    command(
                        "xdotool", "mousemove", "--sync",
                        str(bounds.x + bounds.width // 2),
                        str(bounds.y + bounds.height // 2),
                    )
                    command("xdotool", "click", "1")

                click(settings)
                wait_for(lambda: button("保存配置"), application, "settings form")
                controls = [
                    n for n in walk(app)
                    if n.getState().contains(pyatspi.STATE_EDITABLE)
                ]
                samples = ("http://127.0.0.1:9/v1", "smoke-test-key", "smoke-test-model")
                for name, value in zip(
                    ("https://api.deepseek.com", "sk-…", "deepseek-chat"), samples
                ):
                    field = next(n for n in controls if n.name == name)
                    click(field)
                    command("xdotool", "key", "--window", window_id, "ctrl+a")
                    command("xdotool", "type", "--window", window_id, "--delay", "10", value)
                click(button("保存配置"))
                config_path = root / "config/com.translater.helper/config.json"
                wait_for(
                    lambda: config_path if config_path.exists() else None,
                    application, "saved configuration",
                )
                config = json.loads(config_path.read_text())
                assert (config["baseUrl"], config["apiKey"], config["model"]) == samples
                click(button("返回"))
                wait_for(
                    lambda: find(app, lambda n: n.getState().contains(pyatspi.STATE_EDITABLE)
                                 and n.name.startswith("粘贴任意语言")),
                    application, "home input",
                )
                click(settings)
                wait_for(lambda: button("保存配置"), application, "reopened settings")
                url_field = find(
                    app, lambda n: n.getRoleName() == "entry"
                    and n.name == "https://api.deepseek.com",
                )
                assert url_field.queryText().getText(0, -1) == samples[0]
                print("PASS: AppImage settings mouse input, keyboard input, save, return and reload")
            finally:
                stop(application)
                stop(registry)
                stop(bus)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: linux_appimage_smoke.py APPIMAGE")
    main(Path(sys.argv[1]).resolve(strict=True))
