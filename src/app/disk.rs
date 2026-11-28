use gpui_kit::*;
use crate::host::{host_for, DEFAULT_TIMEOUT};
use crate::metrics::collector::parse_df_partitions;
use super::CrowApp;

impl CrowApp {
    /// Toggles the disk partitions drawer beneath the stat strip.
    pub fn toggle_disk_drawer(&mut self, cx: &mut Context<Self>) {
        self.disk_drawer_open = !self.disk_drawer_open;
        if self.disk_drawer_open {
            let has_partitions = self.fleet.active_server()
                .and_then(|s| self.fleet.metrics_store.get(&s.id))
                .is_some_and(|m| !m.disk_partitions.is_empty());
            if !has_partitions {
                self.refresh_disk_partitions(cx);
            }
        }
        cx.notify();
    }

    /// Asynchronously runs `df -h -P | grep -v tmpfs` on the active server via SSH.
    pub fn refresh_disk_partitions(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        self.disk_drawer_loading = true;
        cx.notify();

        cx.spawn(async move |entity, cx| {
            let target = srv.clone();
            let partitions = cx
                .background_executor()
                .spawn(async move {
                    let host = host_for(&target);
                    let out = host.exec(&["sh", "-c", "df -h -P | grep -v tmpfs"], DEFAULT_TIMEOUT);
                    out.ok().map(|o| parse_df_partitions(&o.stdout)).unwrap_or_default()
                })
                .await;

            let _ = entity.update(cx, |this, cx| {
                this.disk_drawer_loading = false;
                if !partitions.is_empty() {
                    if let Some(m) = this.fleet.metrics_store.get_mut(&srv.id) {
                        m.disk_partitions = partitions.clone();
                    }
                    if let Some(m) = this.fleet.metrics_store.get_mut(&srv.name) {
                        m.disk_partitions = partitions.clone();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
