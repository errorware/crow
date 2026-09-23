use crate::vault::VaultDb;
use crate::keys::{
    expand_tilde, scan_directory, AddScanPathModalState, DiscoveredKey, EditKeyModalState, KeyGenModalState, NewGroupModalState,
    SshKeyGroup, SshKeyRecord, SshScanPath,
};

/// SSH key hub: vault-enrolled keys, groups, scan paths, keys found on disk,
/// and the modals that edit them.
pub struct KeysState {
    pub enrolled: Vec<SshKeyRecord>,
    pub groups: Vec<SshKeyGroup>,
    pub scan_paths: Vec<SshScanPath>,
    pub discovered: Vec<DiscoveredKey>,
    pub group_filter: Option<String>,
    pub scan_status: Option<String>,
    pub gen_modal: Option<KeyGenModalState>,
    pub new_group_modal: Option<NewGroupModalState>,
    pub add_scan_path_modal: Option<AddScanPathModalState>,
    pub edit_modal: Option<EditKeyModalState>,
    pub toast: Option<String>,
}

impl KeysState {
    pub fn new(
        enrolled: Vec<SshKeyRecord>,
        groups: Vec<SshKeyGroup>,
        scan_paths: Vec<SshScanPath>,
        discovered: Vec<DiscoveredKey>,
        scan_status: Option<String>,
    ) -> Self {
        Self {
            enrolled,
            groups,
            scan_paths,
            discovered,
            group_filter: None,
            scan_status,
            gen_modal: None,
            new_group_modal: None,
            add_scan_path_modal: None,
            edit_modal: None,
            toast: None,
        }
    }

    /// Loads keys, groups and scan paths from the vault and scans those paths.
    pub fn load(db: &VaultDb) -> Self {
        let mut st = Self::new(Vec::new(), Vec::new(), Vec::new(), Vec::new(), None);
        st.reload(db);
        st
    }

    /// Re-reads vault records and rescans the scan paths for keys on disk,
    /// leaving modal/filter UI state alone.
    pub fn reload(&mut self, db: &VaultDb) {
        self.scan_paths = db.list_scan_paths().unwrap_or_default();
        self.groups = db.list_key_groups().unwrap_or_default();
        self.enrolled = db.list_ssh_keys().unwrap_or_default();
        let mut discovered: Vec<DiscoveredKey> = Vec::new();
        for p in &self.scan_paths {
            for k in scan_directory(&expand_tilde(&p.path), &self.enrolled) {
                if !discovered.iter().any(|d| d.fingerprint == k.fingerprint) {
                    discovered.push(k);
                }
            }
        }
        let new_count = discovered.iter().filter(|d| !d.is_enrolled).count();
        self.scan_status = Some(format!(
            "Scanned {} path{} · {} key{} found ({} new)",
            self.scan_paths.len(),
            if self.scan_paths.len() == 1 { "" } else { "s" },
            discovered.len(),
            if discovered.len() == 1 { "" } else { "s" },
            new_count
        ));
        self.discovered = discovered;
    }

    pub fn any_modal_open(&self) -> bool {
        self.gen_modal.is_some() || self.new_group_modal.is_some() || self.add_scan_path_modal.is_some() || self.edit_modal.is_some()
    }
}
