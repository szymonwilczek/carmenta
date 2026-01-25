import St from "gi://St";
import Shell from "gi://Shell";
import Meta from "gi://Meta";
import Clutter from "gi://Clutter";
import Gio from "gi://Gio";
import GLib from "gi://GLib";
import * as Main from "resource:///org/gnome/shell/ui/main.js";
import { Extension } from "resource:///org/gnome/shell/extensions/extension.js";

const ServiceIface = `
<node>
  <interface name="org.gnome.Shell.Extensions.Carmenta">
    <method name="Ping">
      <arg type="s" direction="out" name="response" />
    </method>
    <method name="InsertText">
      <arg type="s" direction="in" name="text" />
    </method>
    <method name="PinWindow">
      <arg type="b" direction="in" name="pinned" />
    </method>
  </interface>
</node>`;

export default class CarmentaExtension extends Extension {
  constructor(uuid) {
    super(uuid);
    this._uuid = uuid;
    this._dbus = null;
    this._appId = "io.github.szymonwilczek.carmenta";
    this._lastFocusedWindow = null;
    this._windowFocusId = null;
    this._dbusImpl = null;
    this._ownNameId = null;
    this._virtualDevice = null;
    this._insertTimeoutId = null;
    this._focusTimeoutId = null;
  }

  enable() {
    console.log("Carmenta: Enabling extension");
    this._registerDBus();
    this._registerKeybinding();
    this._trackFocus();

    // virtual keyboard device setup
    this._virtualDevice = Clutter.get_default_backend()
      .get_default_seat()
      .create_virtual_device(
        Clutter.InputDeviceType.KEYBOARD_DEVICE,
        "Carmenta Virtual Keyboard",
      );

    console.log("Carmenta: Extension enabled");
  }

  disable() {
    console.log("Carmenta: Disabling extension");
    this._unregisterDBus();
    this._unregisterKeybinding();
    this._untrackFocus();
    this._dbus = null;

    if (this._insertTimeoutId) {
      GLib.Source.remove(this._insertTimeoutId);
      this._insertTimeoutId = null;
    }

    if (this._focusTimeoutId) {
      GLib.Source.remove(this._focusTimeoutId);
      this._focusTimeoutId = null;
    }

    this._virtualDevice = null;
    this._lastFocusedWindow = null;
  }

  _registerDBus() {
    this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(ServiceIface, this);
    this._dbusImpl.export(
      Gio.DBus.session,
      "/org/gnome/Shell/Extensions/Carmenta",
    );

    this._ownNameId = Gio.bus_own_name(
      Gio.BusType.SESSION,
      "org.gnome.Shell.Extensions.Carmenta",
      Gio.BusNameOwnerFlags.NONE,
      null,
      (connection, name) => {
        console.log(`Carmenta: Acquired name ${name}`);
      },
      (connection, name) => {
        console.log(`Carmenta: Lost name ${name}`);
      },
    );
  }

  _unregisterDBus() {
    if (this._dbusImpl) {
      this._dbusImpl.unexport();
      this._dbusImpl = null;
    }
    if (this._ownNameId) {
      Gio.bus_unown_name(this._ownNameId);
      this._ownNameId = null;
    }
  }

  _registerKeybinding() {
    Main.wm.addKeybinding(
      "carmenta-shortcut",
      this.getSettings(),
      Meta.KeyBindingFlags.NONE,
      Shell.ActionMode.ALL,
      () => {
        this._spawnApp();
      },
    );
  }

  _unregisterKeybinding() {
    Main.wm.removeKeybinding("carmenta-shortcut");
  }

  _trackFocus() {
    this._windowFocusId = global.display.connect("notify::focus-window", () => {
      let win = global.display.focus_window;
      // ignore our own window
      if (win) {
        const wmClass = win.get_wm_class();
        if (wmClass && !wmClass.toLowerCase().includes("carmenta")) {
          this._lastFocusedWindow = win;
        }
      }
    });
  }

  _untrackFocus() {
    if (this._windowFocusId) {
      global.display.disconnect(this._windowFocusId);
      this._windowFocusId = null;
    }
  }

  Ping() {
    return "Pong";
  }

  PinWindow(pinned) {
    // find carmenta window
    let carmentaWin = this._findCarmentaWindow();
    if (carmentaWin) {
      if (pinned) {
        carmentaWin.make_above();
        carmentaWin.stick(); // visible on all workspaces
        console.log("Carmenta: Window set to ALWAYS ON TOP + STICKY");
      } else {
        carmentaWin.unmake_above();
        carmentaWin.unstick();
      }
    } else {
      console.log("Carmenta: Window not found for pinning");
    }
  }

  InsertText(text) {
    console.log(`Carmenta: Injecting text '${text}'`);

    if (this._lastFocusedWindow) {
      // activate target
      this._lastFocusedWindow.activate(global.get_current_time());

      // copy and paste
      if (this._insertTimeoutId) {
        GLib.Source.remove(this._insertTimeoutId);
        this._insertTimeoutId = null;
      }

      this._insertTimeoutId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1, () => {
        this._copyToClipboard(text);
        this._sendCtrlV();
        this._insertTimeoutId = null;

        // return focus to carmenta
        if (this._focusTimeoutId) {
          GLib.Source.remove(this._focusTimeoutId);
          this._focusTimeoutId = null;
        }

        this._focusTimeoutId = GLib.timeout_add(
          GLib.PRIORITY_DEFAULT,
          10,
          () => {
            let carmentaWin = this._findCarmentaWindow();
            if (carmentaWin) {
              carmentaWin.activate(global.get_current_time());
              carmentaWin.make_above();
              console.log("Carmenta: Focus returned");
            }
            this._focusTimeoutId = null;
            return GLib.SOURCE_REMOVE;
          },
        );

        return GLib.SOURCE_REMOVE;
      });
    } else {
      console.log("Carmenta: No last focused window found");
      this._copyToClipboard(text);
    }
  }

  _findCarmentaWindow() {
    let windows = global.display.get_tab_list(Meta.TabList.NORMAL, null);
    return windows.find((w) => {
      let wmClass = w.get_wm_class();
      return wmClass && wmClass.toLowerCase().includes("carmenta");
    });
  }

  _copyToClipboard(text) {
    const clipboard = St.Clipboard.get_default();
    clipboard.set_text(St.ClipboardType.CLIPBOARD, text);
  }

  _sendCtrlV() {
    if (!this._virtualDevice) return;

    const time = Clutter.get_current_event_time();

    // Ctrl down
    this._virtualDevice.notify_keyval(
      time,
      Clutter.KEY_Control_L,
      Clutter.KeyState.PRESSED,
    );
    // V down
    this._virtualDevice.notify_keyval(
      time,
      Clutter.KEY_v,
      Clutter.KeyState.PRESSED,
    );
    // V up
    this._virtualDevice.notify_keyval(
      time,
      Clutter.KEY_v,
      Clutter.KeyState.RELEASED,
    );
    // Ctrl up
    this._virtualDevice.notify_keyval(
      time,
      Clutter.KEY_Control_L,
      Clutter.KeyState.RELEASED,
    );
  }

  _spawnApp() {
    try {
      console.log("Carmenta: Launching app via keybinding");
      const launcher = new Gio.SubprocessLauncher({
        flags: Gio.SubprocessFlags.NONE,
      });

      // try launching from PATH first
      try {
        launcher.spawnv(["carmenta"]);
        console.log("Carmenta: App launched successfully");
      } catch (pathError) {
        // try common installation locations
        const locations = [
          GLib.get_home_dir() + "/.cargo/bin/carmenta",
          GLib.get_home_dir() + "/.local/bin/carmenta",
          "/usr/local/bin/carmenta",
          "/usr/bin/carmenta",
        ];

        let launched = false;
        for (const path of locations) {
          if (GLib.file_test(path, GLib.FileTest.EXISTS)) {
            try {
              launcher.spawnv([path]);
              console.log(`Carmenta: App launched from ${path}`);
              launched = true;
              break;
            } catch (e) {
              log(`[Carmenta] Failed to launch ${path}: ${e}`);
            }
          }
        }

        if (launched) {
          return; // exit if successful
        }

        try {
          let flatpakLauncher = new Gio.SubprocessLauncher({
            flags:
              Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_PIPE,
          });
          flatpakLauncher.spawnv(["flatpak", "run", "org.carmenta.App"]);
          log("[Carmenta] Launched via Flatpak");
          return; // exit if successful
        } catch (e) {
          log(`[Carmenta] Failed to launch flatpak: ${e}`);
        }

        Main.notify(
          "Carmenta",
          "Could not find 'carmenta' executable in PATH or Flatpak.",
        );
      }
    } catch (e) {
      console.error(`Carmenta: Failed to launch app: ${e}`);
    }
  }
}
