use gpui_kit::*;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, IntoElement)]
pub enum TablerIcon {
    // Navigation / Views
    LayoutDashboard,
    Cpu,
    Server,
    Box,
    AdjustmentsHorizontal,
    FileText,
    Photo,
    Folder,
    Clock,
    Users,
    ShieldCheck,
    Terminal2,
    LayoutColumns,
    LayoutRows,
    Network,
    Database,

    // Window & Chrome Controls
    Minus,
    Square,
    X,
    Plus,

    // Status / Feedback
    Check,
    AlertTriangle,
    AlertCircle,
    InfoCircle,

    // Operations / Actions
    Refresh,
    PlayerPlay,
    PlayerStop,
    PlayerPause,
    Power,
    Trash,
    Copy,
    Search,
    Filter,
    Dots,
    DotsVertical,
    ChevronRight,
    ChevronDown,
    LayoutSidebarLeftCollapse,
    LayoutSidebarLeftExpand,
    Key,
    Lock,
    Activity,
    Settings,
    Bell,
    ExternalLink,
    Download,
    Upload,
    GitBranch,
    GitCommit,
    Cloud,
}

impl TablerIcon {
    pub const fn bytes(self) -> &'static [u8] {
        match self {
            Self::LayoutDashboard => include_bytes!("../../assets/icons/layout-dashboard.svg"),
            Self::Cpu => include_bytes!("../../assets/icons/cpu.svg"),
            Self::Server => include_bytes!("../../assets/icons/server.svg"),
            Self::Box => include_bytes!("../../assets/icons/box.svg"),
            Self::AdjustmentsHorizontal => include_bytes!("../../assets/icons/adjustments-horizontal.svg"),
            Self::FileText => include_bytes!("../../assets/icons/file-text.svg"),
            Self::Photo => include_bytes!("../../assets/icons/photo.svg"),
            Self::Folder => include_bytes!("../../assets/icons/folder.svg"),
            Self::Clock => include_bytes!("../../assets/icons/clock.svg"),
            Self::Users => include_bytes!("../../assets/icons/users.svg"),
            Self::ShieldCheck => include_bytes!("../../assets/icons/shield-check.svg"),
            Self::Terminal2 => include_bytes!("../../assets/icons/terminal-2.svg"),
            Self::LayoutColumns => include_bytes!("../../assets/icons/layout-columns.svg"),
            Self::LayoutRows => include_bytes!("../../assets/icons/layout-rows.svg"),
            Self::Network => include_bytes!("../../assets/icons/network.svg"),
            Self::Database => include_bytes!("../../assets/icons/database.svg"),

            Self::Minus => include_bytes!("../../assets/icons/minus.svg"),
            Self::Square => include_bytes!("../../assets/icons/square.svg"),
            Self::X => include_bytes!("../../assets/icons/x.svg"),
            Self::Plus => include_bytes!("../../assets/icons/plus.svg"),

            Self::Check => include_bytes!("../../assets/icons/check.svg"),
            Self::AlertTriangle => include_bytes!("../../assets/icons/alert-triangle.svg"),
            Self::AlertCircle => include_bytes!("../../assets/icons/alert-circle.svg"),
            Self::InfoCircle => include_bytes!("../../assets/icons/info-circle.svg"),

            Self::Refresh => include_bytes!("../../assets/icons/refresh.svg"),
            Self::PlayerPlay => include_bytes!("../../assets/icons/player-play.svg"),
            Self::PlayerStop => include_bytes!("../../assets/icons/player-stop.svg"),
            Self::PlayerPause => include_bytes!("../../assets/icons/player-pause.svg"),
            Self::Power => include_bytes!("../../assets/icons/power.svg"),
            Self::Trash => include_bytes!("../../assets/icons/trash.svg"),
            Self::Copy => include_bytes!("../../assets/icons/copy.svg"),
            Self::Search => include_bytes!("../../assets/icons/search.svg"),
            Self::Filter => include_bytes!("../../assets/icons/filter.svg"),
            Self::Dots => include_bytes!("../../assets/icons/dots.svg"),
            Self::DotsVertical => include_bytes!("../../assets/icons/dots-vertical.svg"),
            Self::ChevronRight => include_bytes!("../../assets/icons/chevron-right.svg"),
            Self::ChevronDown => include_bytes!("../../assets/icons/chevron-down.svg"),
            Self::LayoutSidebarLeftCollapse => include_bytes!("../../assets/icons/layout-sidebar-left-collapse.svg"),
            Self::LayoutSidebarLeftExpand => include_bytes!("../../assets/icons/layout-sidebar-left-expand.svg"),
            Self::Key => include_bytes!("../../assets/icons/key.svg"),
            Self::Lock => include_bytes!("../../assets/icons/lock.svg"),
            Self::Activity => include_bytes!("../../assets/icons/activity.svg"),
            Self::Settings => include_bytes!("../../assets/icons/settings.svg"),
            Self::Bell => include_bytes!("../../assets/icons/bell.svg"),
            Self::ExternalLink => include_bytes!("../../assets/icons/external-link.svg"),
            Self::Download => include_bytes!("../../assets/icons/download.svg"),
            Self::Upload => include_bytes!("../../assets/icons/upload.svg"),
            Self::GitBranch => include_bytes!("../../assets/icons/git-branch.svg"),
            Self::GitCommit => include_bytes!("../../assets/icons/git-commit.svg"),
            Self::Cloud => include_bytes!("../../assets/icons/cloud.svg"),
        }
    }
}

pub fn tabler_icon(icon: TablerIcon) -> Svg {
    svg()
        .data(icon.bytes())
        .flex_shrink_0()
}

/// An icon that takes its colour (hover colours included) from the text
/// around it, like a glyph would. `tabler_icon` needs an explicit
/// `.text_color(...)`: an svg doesn't inherit the parent's text colour and
/// is invisible without one.
pub fn inherited_icon(icon: TablerIcon, size: Pixels) -> Div {
    div().flex_none().flex().items_center().text_size(size).child(icon)
}

impl RenderOnce for TablerIcon {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let text_style = window.text_style();
        tabler_icon(self)
            .size(text_style.font_size.to_pixels(window.rem_size()))
            .text_color(text_style.color)
    }
}
