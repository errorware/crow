//! W00: back to the start, so W01–W09 can run in order on a fresh profile.
//! The servers trust the test key (and the harness's own) again, Crow's
//! drop-ins and the bastion-only rule are gone, app1's firewall is off; the
//! profile is emptied except the test key.

use gpui_kit::TestAppContext;

use super::harness::{ensure_outside_key, root, ssh, ssh_via, step, targets};

#[gpui_kit::test]
fn w00_reset(_cx: &mut TestAppContext) {
    let Some(root) = root() else { return };
    let t = targets();
    let all = [t["BASTION"].as_str(), t["APP1"].as_str(), t["APP2"].as_str()];
    let test_pub = std::fs::read_to_string(root.join("home/.ssh/id_ed25519.pub")).unwrap();

    step("The profile: only the test key, the harness's key and targets.env stay");
    for d in ["xdg", "home/.config"] {
        let _ = std::fs::remove_dir_all(root.join(d));
    }
    let _ = std::fs::remove_dir_all("/tmp/crow-wf-run");
    for entry in std::fs::read_dir(root.join("home/.ssh")).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name != "id_ed25519" && name != "id_ed25519.pub" {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    step("The harness's own key on every server (with whichever key still works)");
    ensure_outside_key(&all);
    let outside_pub = std::fs::read_to_string(root.join("outside_ed25519.pub")).unwrap();

    for addr in all {
        step(&format!("{addr}: trust only the test key and the harness's; undo Crow's sshd and firewall changes"));
        let script = format!(
            "printf '%s\\n%s\\n' '{}' '{}' > /root/.ssh/authorized_keys; chmod 600 /root/.ssh/authorized_keys; \
             rm -f /etc/ssh/sshd_config.d/00-crow-no-password.conf /etc/ssh/sshd_config.d/00-crow-root-login.conf /etc/ssh/sshd_config.d/10-crow-wf.conf; \
             sed -i '/^ClientAliveInterval 120$/d' /etc/ssh/sshd_config; \
             command -v ufw >/dev/null && ufw --force disable >/dev/null; \
             userdel -r wfmarta 2>/dev/null; rm -rf /etc/crow-wf; sed -i '/crow-wf-drift/d' /etc/hosts; \
             sshd -t && systemctl reload sshd 2>/dev/null || systemctl reload ssh; echo reset",
            test_pub.trim(),
            outside_pub.trim()
        );
        // A run that stopped half way may have left app2 bastion-only.
        let (mut ok, mut out) = ssh(addr, &script);
        if !ok {
            (ok, out) = ssh_via(&t["BASTION"], addr, &script);
        }
        assert!(ok && out.contains("reset"), "{addr}: {out}");
    }

}
