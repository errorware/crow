//! W05: user accounts. Create one with sudo and a password (Ubuntu's sudo
//! group, Rocky's wheel), lock and unlock it, delete it; and Crow won't
//! delete the account it logs in as.

use gpui_kit::TestAppContext;

use super::harness::{act, isolate, open_crow, root, server, settle, ssh, step, targets};

const USER: &str = "wfmarta";

#[gpui_kit::test]
fn w05_user_accounts(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    isolate();
    let t = targets();
    let (app, w) = open_crow(cx);
    settle(cx, w);

    for (name, addr, sudo_group) in [("wf-app1", t["APP1"].clone(), "sudo"), ("wf-app2", t["APP2"].clone(), "wheel")] {
        let _ = ssh(&addr, &format!("userdel -r {USER} 2>/dev/null; true"));
        let srv = server(cx, &app, name);
        step(&format!("{name} → Users: the accounts are read"));
        act(cx, &app, w, |this, _, cx| {
            this.switch_tab(&srv.id, cx);
            this.set_view("users", cx);
            this.refresh_users(cx);
        });
        let (count, err, group) = app.read_with(cx, |this, _| (this.users.users.len(), this.users.load_error.clone(), this.users.sudo_group()));
        assert!(count > 5 && err.is_none(), "{count} {err:?}");
        assert_eq!(group, Some(sudo_group), "the distro's sudo group");

        step(&format!("New user {USER}: sudo, a password"));
        act(cx, &app, w, |this, _, cx| {
            this.open_new_user_modal(cx);
            let f = &mut this.users.new_user;
            f.username = USER.into();
            f.gecos = "Marta (workflow)".into();
            f.grant_sudo = true;
            f.password = crate::secret_string::SecretString::new("wf-Marta-pass-91");
            f.password_confirm = crate::secret_string::SecretString::new("wf-Marta-pass-91");
            this.user_create(cx);
        });
        let (toast, found) = app.read_with(cx, |this, _| (this.users.toast.clone().unwrap_or_default(), this.users.find(USER).cloned()));
        let u = found.unwrap_or_else(|| panic!("{USER} is listed after creating: {toast}"));
        assert!(u.groups.iter().any(|g| g == sudo_group), "in {sudo_group}: {:?} ({toast})", u.groups);
        let (_, out) = ssh(&addr, &format!("id {USER}; getent shadow {USER} | cut -d: -f2 | cut -c1-3; test -d /home/{USER} && echo home"));
        assert!(out.contains(sudo_group) && out.contains("home") && (out.contains("$y$") || out.contains("$6$")), "a hashed password and a home: {out}");

        step("Lock, then unlock");
        act(cx, &app, w, |this, _, cx| this.user_toggle_lock(USER, cx));
        assert!(app.read_with(cx, |this, _| this.users.find(USER).is_some_and(|u| u.is_locked)), "locked");
        let (_, out) = ssh(&addr, &format!("passwd -S {USER}"));
        assert!(out.split_whitespace().nth(1).is_some_and(|s| s == "L" || s == "LK"), "{out}");
        act(cx, &app, w, |this, _, cx| this.user_toggle_lock(USER, cx));
        assert!(app.read_with(cx, |this, _| this.users.find(USER).is_some_and(|u| !u.is_locked)), "unlocked");

        step("Crow won't delete root, the account it logs in as");
        act(cx, &app, w, |this, _, cx| this.user_delete_clicked("root", cx));
        let toast = app.read_with(cx, |this, _| this.users.toast.clone().unwrap_or_default());
        assert!(toast.contains("lock Crow"), "{toast}");
        assert!(ssh(&addr, "true").0);

        step(&format!("Delete {USER}: armed by the first click, done by the second"));
        act(cx, &app, w, |this, _, cx| this.user_delete_clicked(USER, cx));
        assert!(app.read_with(cx, |this, _| this.users.find(USER).is_some()), "not yet");
        act(cx, &app, w, |this, _, cx| this.user_delete_clicked(USER, cx));
        assert!(app.read_with(cx, |this, _| this.users.find(USER).is_none()), "gone from the list");
        let (exists, _) = ssh(&addr, &format!("id {USER}"));
        assert!(!exists, "gone from the server");
    }
}
