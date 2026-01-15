import St from "gi://St";
import Shell from "gi://Shell";
import Meta from "gi://Meta";
import Clutter from "gi://Clutter";
import Gio from "gi://Gio";
import GLib from "gi://GLib";
import * as Main from "resource:///org/gnome/shell/ui/main.js";
import { Extension } from "resource:///org/gnome/shell/extensions/extension.js";

const DBusInterface = `
<node>
  <interface name="org.gnome.Shell.Extensions.Carmenta">
    <method name="Ping">
      <arg type="s" direction="out" name="response" />
    </method>
    <method name="InsertText">
        <arg type="s" direction="in" name="text" />
    </method>
  </interface>
</node>`;

export default class CarmentaExtension extends Extension {
  enable() {
    this._lastFocusedWindow = null;
    this._windowFocusId = global.display.connect("notify::focus-window", () => {
      let win = global.display.focus_window;
      // Ignorujemy nasze własne okno (zakładamy, że WM_CLASS/AppID to 'org.carmenta.App' lub 'Carmenta')
      if (win) {
        const wmClass = win.get_wm_class();
        // TODO: Sprawdzić jaki dokładnie wm_class ma apka.
        // Na razie, jeśli to NIE jest Carmenta, to zapamiętujemy.
        if (wmClass && !wmClass.toLowerCase().includes("carmenta")) {
          this._lastFocusedWindow = win;
          // console.log(`Carmenta: Tracking focus: ${wmClass}`);
        }
      }
    });

    this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(DBusInterface, this);
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

    // Virtual Keyboard device setup
    this._virtualDevice = Clutter.get_default_backend()
      .get_default_seat()
      .create_virtual_device(
        Clutter.InputDeviceType.KEYBOARD_DEVICE,
        "Carmenta Virtual Keyboard",
      );

    console.log("Carmenta: Extension enabled");

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

  disable() {
    if (this._windowFocusId) {
      global.display.disconnect(this._windowFocusId);
      this._windowFocusId = null;
    }

    if (this._ownNameId) {
      Gio.bus_unown_name(this._ownNameId);
      this._ownNameId = null;
    }

    if (this._dbusImpl) {
      this._dbusImpl.unexport();
      this._dbusImpl = null;
    }

    // Virtual devices are destroyed automatically when seat is disposed,
    // but explicitly clearing reference is good.
    this._virtualDevice = null;
    this._lastFocusedWindow = null;

    Main.wm.removeKeybinding("carmenta-shortcut");
  }

  Ping() {
    return "Pong";
  }

  InsertText(text) {
    console.log(`Carmenta: Injecting text: ${text}`);

    // 1. Aktywuj poprzednie okno
    if (this._lastFocusedWindow) {
      Main.activateWindow(this._lastFocusedWindow);
    } else {
      console.warn("Carmenta: No last window found to paste into!");
    }

    // 2. Skopiuj do schowka
    const clipboard = St.Clipboard.get_default();
    clipboard.set_text(St.ClipboardType.CLIPBOARD, text);

    // 3. Wyślij Ctrl+V (z małym opóźnieniem, żeby okno zdążyło dostać fokus)
    GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
      this._sendCtrlV();
      return GLib.SOURCE_REMOVE;
    });
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
      // Tymczasowo logujemy
      console.log("Carmenta: Shortcut pressed!");
      Main.notify("Carmenta", "Shortcut pressed!");

      // Tutaj docelowo spawnujemy proces
    } catch (e) {
      console.error(e);
    }
  }
}
