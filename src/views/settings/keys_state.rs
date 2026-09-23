use crate::keys::{
    AddScanPathModalState, DiscoveredKey, EditKeyModalState, KeyGenModalState, NewGroupModalState,
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

    pub fn any_modal_open(&self) -> bool {
        self.gen_modal.is_some() || self.new_group_modal.is_some() || self.add_scan_path_modal.is_some() || self.edit_modal.is_some()
    }
}
