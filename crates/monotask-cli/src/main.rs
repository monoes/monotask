use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "monotaskcli", about = "Monotask – P2P task manager CLI")]
struct Cli {
    #[arg(long, global = true, help = "Data directory")]
    data_dir: Option<std::path::PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize identity and config
    Init,
    /// Board management
    Board {
        #[command(subcommand)]
        cmd: BoardCommands,
    },
    /// Column management
    Column {
        #[command(subcommand)]
        cmd: ColumnCommands,
    },
    /// Card management
    Card {
        #[command(subcommand)]
        cmd: CardCommands,
    },
    /// Checklist management
    Checklist {
        #[command(subcommand)]
        cmd: ChecklistCommands,
    },
    /// Manage Spaces (shared containers for boards)
    Space {
        #[command(subcommand)]
        cmd: SpaceCommands,
    },
    /// Manage your local identity and profile
    Profile {
        #[command(subcommand)]
        cmd: ProfileCommands,
    },
    /// Print version
    Version,
    /// Print full reference documentation for AI agents and automation
    #[command(name = "ai-help")]
    AiHelp,
    /// GitHub Issues integration
    Github {
        #[command(subcommand)]
        cmd: GithubCommands,
    },
    /// Linear Issues integration
    Linear {
        #[command(subcommand)]
        cmd: LinearCommands,
    },
    /// Gmail / Outlook email integration (CRM)
    Mail {
        #[command(subcommand)]
        cmd: MailCommands,
    },
    /// Manage board-level custom field definitions
    Field {
        #[command(subcommand)]
        cmd: FieldCommands,
    },
    /// Open a monotask:// deep link in the desktop app
    App {
        #[command(subcommand)]
        cmd: AppCommands,
    },
    /// Space chat — send and list messages in a space chat thread
    Chat {
        #[command(subcommand)]
        cmd: ChatCommands,
    },
    /// Start P2P sync daemon
    Sync {
        /// Run in background (writes PID to data dir)
        #[arg(long)]
        detach: bool,
        /// Stop background daemon
        #[arg(long)]
        stop: bool,
        /// Show sync status
        #[arg(long)]
        status: bool,
        /// TCP port to listen on (default: OS-assigned). Use a fixed port when
        /// peers need to dial you directly via --peer.
        #[arg(long, default_value_t = 0)]
        port: u16,
        /// Dial a specific peer at startup (bypasses mDNS). Format:
        /// /ip4/1.2.3.4/tcp/7272  — repeat for multiple peers.
        #[arg(long = "peer")]
        peers: Vec<String>,
    },
}

#[derive(Subcommand)]
enum BoardCommands {
    /// Create a new board inside a space. Boards must belong to a space.
    ///
    /// SPACE_ID is required. Run `monotaskcli space list` to see your spaces.
    Create {
        title: String,
        /// Space the board belongs to (required)
        #[arg(long, value_name = "SPACE_ID")]
        space: String,
        #[arg(long)]
        json: bool,
    },
    /// List all boards with their titles (--json returns [{id, title}])
    List { #[arg(long)] json: bool },
    /// Rename a board
    Rename { board_id: String, new_title: String, #[arg(long)] json: bool },
    /// Permanently delete a board and remove it from its space
    Delete {
        board_id: String,
        /// Space the board belongs to (required)
        #[arg(long, value_name = "SPACE_ID")]
        space: String,
        #[arg(long)]
        json: bool,
    },
    /// Show board schema: columns and custom field definitions
    Schema { board_id: String, #[arg(long)] json: bool },
    /// Undo the most recent mutation on a board (restores previous snapshot)
    Undo { board_id: String, #[arg(long)] json: bool },
    /// Redo the most recently undone mutation on a board
    Redo { board_id: String, #[arg(long)] json: bool },
}

#[derive(Subcommand)]
enum ColumnCommands {
    /// Create a new column in a board
    Create { board_id: String, title: String, #[arg(long)] json: bool },
    /// List all columns in a board with their card IDs
    List { board_id: String, #[arg(long)] json: bool },
    /// Rename a column
    Rename { board_id: String, col_id: String, new_title: String, #[arg(long)] json: bool },
    /// Delete a column
    Delete { board_id: String, col_id: String, #[arg(long)] json: bool },
}

#[derive(Subcommand)]
enum CardCommands {
    /// Create a card in a column
    Create {
        board_id: String,
        col_id: String,
        title: String,
        /// Set custom field values at creation (FIELD_NAME_OR_UUID=VALUE, repeat for multiple)
        #[arg(long = "field")]
        fields: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// List all non-deleted, non-archived cards. Filter with --col, --label, and/or --where.
    List {
        board_id: String,
        /// Only return cards in this column ID
        #[arg(long)]
        col: Option<String>,
        /// Only return cards that have this label (exact match)
        #[arg(long)]
        label: Option<String>,
        /// Filter by custom field (FIELD_REF=VALUE or FIELD_REF~SUBSTRING, repeat for AND)
        #[arg(long = "where")]
        filters: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show all fields of a card including parent and subtasks
    View { board_id: String, card_id: String, #[arg(long)] json: bool },
    /// Rename a card
    Rename { board_id: String, card_id: String, new_title: String, #[arg(long)] json: bool },
    /// Soft-delete a card (hidden from all views)
    Delete { board_id: String, card_id: String, #[arg(long)] json: bool },
    /// Soft-archive a card (hidden from normal views)
    Archive { board_id: String, card_id: String, #[arg(long)] json: bool },
    /// Copy a card into another column on the same board
    Copy { board_id: String, card_id: String, col_id: String, #[arg(long)] json: bool },
    /// Move a card to a different column (auto-detects current column)
    Move { board_id: String, card_id: String, to_col_id: String, #[arg(long)] json: bool },
    /// Set the card's long-form description (markdown supported)
    SetDescription { board_id: String, card_id: String, text: String, #[arg(long)] json: bool },
    /// Set the card cover color. Use "none" to clear
    SetCover { board_id: String, card_id: String, color: String, #[arg(long)] json: bool },
    /// Set due date (YYYY-MM-DD). Use "none" to clear
    SetDueDate { board_id: String, card_id: String, date: String, #[arg(long)] json: bool },
    /// Set a legacy string priority label. Use "none" to clear
    SetPriority { board_id: String, card_id: String, priority: String, #[arg(long)] json: bool },
    /// Set impact score (0–10). Priority = floor((impact + 10 - effort) / 2)
    SetImpact { board_id: String, card_id: String, #[arg(value_parser = clap::value_parser!(u8).range(0..=10))] value: u8, #[arg(long)] json: bool },
    /// Set effort score (0–10). Priority = floor((impact + 10 - effort) / 2)
    SetEffort { board_id: String, card_id: String, #[arg(value_parser = clap::value_parser!(u8).range(0..=10))] value: u8, #[arg(long)] json: bool },
    /// Set direct priority (0–10) when impact and effort are not used. Use --clear to remove.
    SetDirectPriority { board_id: String, card_id: String, #[arg(value_parser = clap::value_parser!(u8).range(0..=10), conflicts_with = "clear")] value: Option<u8>, #[arg(long)] clear: bool, #[arg(long)] json: bool },
    /// Clear impact, effort, and direct priority — resets scoring to unset state
    ClearPriority { board_id: String, card_id: String, #[arg(long)] json: bool },
    /// Assign a card to a user (hex pubkey). Use "none" to clear
    SetAssignee { board_id: String, card_id: String, pubkey: String, #[arg(long)] json: bool },
    /// Attach an image file to a card (stored as base64; referenced as img:<id> in markdown)
    AttachImage { board_id: String, card_id: String, file: String, #[arg(long)] json: bool },
    /// List all attachments on a card
    ListAttachments { board_id: String, card_id: String, #[arg(long)] json: bool },
    /// Save an attachment to a file
    SaveAttachment { board_id: String, card_id: String, attachment_id: String, #[arg(long)] output: Option<String>, #[arg(long)] json: bool },
    /// Label management
    Label {
        #[command(subcommand)]
        cmd: LabelCommands,
    },
    /// Comment management
    Comment {
        #[command(subcommand)]
        cmd: CommentCommands,
    },
    /// Subtask management
    Subtask {
        #[command(subcommand)]
        cmd: SubtaskCommands,
    },
    /// Link management (card-to-card references)
    Link {
        #[command(subcommand)]
        cmd: LinkCommands,
    },
    /// List @mentions found in a card's description
    Mentions {
        board_id: String,
        card_id: String,
        #[arg(long)] json: bool,
    },
    /// Prerequisite management
    Prerequisite {
        #[command(subcommand)]
        cmd: PrerequisiteCommands,
    },
    /// Set a custom field value on a card. FIELD_REF may be field name or UUID.
    FieldSet {
        board_id: String,
        card_id: String,
        /// Field name or UUID
        field_ref: String,
        value: String,
        #[arg(long)] json: bool,
    },
    /// Get the value of a custom field on a card
    FieldGet {
        board_id: String,
        card_id: String,
        /// Field name or UUID
        field_ref: String,
        #[arg(long)] json: bool,
    },
    /// Clear a custom field value from a card
    FieldClear {
        board_id: String,
        card_id: String,
        /// Field name or UUID
        field_ref: String,
        #[arg(long)] json: bool,
    },
    /// List all custom field values on a card with resolved field names
    FieldList {
        board_id: String,
        card_id: String,
        #[arg(long)] json: bool,
    },
    /// Create-or-update a card matching a custom field value (CRM upsert)
    Upsert {
        board_id: String,
        col_id: String,
        title: String,
        /// Field to match on (name or UUID) when searching for an existing card
        #[arg(long)]
        match_field: String,
        /// Value the match-field must equal to count as an update
        #[arg(long)]
        match_value: String,
        /// Set field values (FIELD_NAME_OR_UUID=VALUE, repeat for multiple)
        #[arg(long = "field")]
        fields: Vec<String>,
        #[arg(long)] json: bool,
    },
}

#[derive(Subcommand)]
enum PrerequisiteCommands {
    /// Mark an existing card as a prerequisite of another card
    Add {
        /// Board of the dependent card
        board_id: String,
        /// Dependent card ID (the card that requires the prerequisite)
        card_id: String,
        /// Board of the prerequisite card
        prereq_board_id: String,
        /// Prerequisite card ID
        prereq_card_id: String,
        #[arg(long)] json: bool,
    },
    /// List prerequisites of a card
    List {
        board_id: String,
        card_id: String,
        #[arg(long)] json: bool,
    },
    /// Remove a prerequisite link from a card
    Remove {
        board_id: String,
        card_id: String,
        prereq_board_id: String,
        prereq_card_id: String,
        #[arg(long)] json: bool,
    },
}

#[derive(Subcommand)]
enum LinkCommands {
    /// Add a card-to-card link
    Add {
        board_id: String,
        card_id: String,
        /// Board containing the target card
        target_board_id: String,
        target_card_id: String,
        #[arg(long)] json: bool,
    },
    /// List all card links on a card
    List {
        board_id: String,
        card_id: String,
        #[arg(long)] json: bool,
    },
    /// Remove a card-to-card link
    Remove {
        board_id: String,
        card_id: String,
        target_board_id: String,
        target_card_id: String,
        #[arg(long)] json: bool,
    },
}

#[derive(Subcommand)]
enum SubtaskCommands {
    /// Add a subtask (creates a new card linked as a child of the given card)
    Add {
        /// Board that owns the parent card
        parent_board_id: String,
        /// Parent card ID
        parent_card_id: String,
        /// Board where the subtask card will be created (defaults to same as parent)
        child_board_id: String,
        /// Column ID in the child board
        col_id: String,
        /// Title for the new subtask card
        title: String,
        #[arg(long)] json: bool,
    },
    /// List subtasks of a card
    List {
        board_id: String,
        card_id: String,
        #[arg(long)] json: bool,
    },
}

#[derive(Subcommand)]
enum LabelCommands {
    Add { board_id: String, card_id: String, label: String, #[arg(long)] json: bool },
    Remove { board_id: String, card_id: String, label: String, #[arg(long)] json: bool },
    List { board_id: String, card_id: String, #[arg(long)] json: bool },
}

#[derive(Subcommand)]
enum CommentCommands {
    /// Add a comment to a card
    Add {
        board_id: String,
        card_id: String,
        text: String,
        #[arg(long)] json: bool,
    },
    /// List comments on a card
    List {
        board_id: String,
        card_id: String,
        #[arg(long)] json: bool,
    },
    /// Delete a comment
    Delete {
        board_id: String,
        card_id: String,
        comment_id: String,
        #[arg(long)] json: bool,
    },
    /// Edit a comment
    Edit {
        board_id: String,
        card_id: String,
        comment_id: String,
        new_text: String,
        #[arg(long)] json: bool,
    },
}

#[derive(Subcommand)]
enum ChecklistCommands {
    /// Add a checklist to a card
    Add {
        board_id: String,
        card_id: String,
        title: String,
        #[arg(long)] json: bool,
    },
    /// Add an item to a checklist
    ItemAdd {
        board_id: String,
        card_id: String,
        checklist_id: String,
        text: String,
        #[arg(long)] json: bool,
    },
    /// Check a checklist item
    ItemCheck {
        board_id: String,
        card_id: String,
        checklist_id: String,
        item_id: String,
        #[arg(long)] json: bool,
    },
    /// Uncheck a checklist item
    ItemUncheck {
        board_id: String,
        card_id: String,
        checklist_id: String,
        item_id: String,
        #[arg(long)] json: bool,
    },
    /// Delete a checklist item
    ItemDelete {
        board_id: String,
        card_id: String,
        checklist_id: String,
        item_id: String,
        #[arg(long)] json: bool,
    },
    /// Delete a checklist
    Delete {
        board_id: String,
        card_id: String,
        checklist_id: String,
        #[arg(long)] json: bool,
    },
}

#[derive(clap::Subcommand)]
enum SpaceCommands {
    /// Create a new Space
    Create { name: String },
    /// List all local Spaces
    List,
    /// Show details of a Space
    Info { space_id: String },
    /// Manage invite tokens for a Space
    Invite {
        #[command(subcommand)]
        cmd: SpaceInviteCommands,
    },
    /// Join a Space via a token or .space file
    Join { token_or_file: String },
    /// Manage boards associated with a Space
    Boards {
        #[command(subcommand)]
        cmd: SpaceBoardsCommands,
    },
    /// Manage members of a Space
    Members {
        #[command(subcommand)]
        cmd: SpaceMembersCommands,
    },
}

#[derive(clap::Subcommand)]
enum SpaceInviteCommands {
    /// Generate a new invite token for a Space
    Generate { space_id: String },
    /// Export an invite as a .space file
    Export { space_id: String, output_file: String },
    /// Revoke all active invites for a Space
    Revoke { space_id: String },
}

#[derive(clap::Subcommand)]
enum SpaceBoardsCommands {
    /// Add a board to a Space
    Add { space_id: String, board_id: String },
    /// Remove a board from a Space
    Remove { space_id: String, board_id: String },
    /// List boards in a Space
    List { space_id: String },
}

#[derive(clap::Subcommand)]
enum SpaceMembersCommands {
    /// List members of a Space
    List { space_id: String },
    /// Kick a member from a Space
    Kick { space_id: String, pubkey: String },
}

#[derive(clap::Subcommand)]
enum ProfileCommands {
    /// Show your current profile
    Show,
    /// Set your display name
    SetName { name: String },
    /// Set your avatar from an image file
    SetAvatar { path: String },
    /// Import an SSH Ed25519 key as your identity
    ImportSshKey { path: Option<String> },
}

#[derive(Subcommand)]
enum GithubCommands {
    /// Save a GitHub Personal Access Token (PAT)
    Connect {
        /// The PAT token (ghp_…). Reads from stdin if not given.
        token: Option<String>,
    },
    /// Show whether a token is saved (no network check)
    Status,
    /// Link a board to a GitHub repository
    Link {
        board_id: String,
        /// GitHub owner (user or org)
        owner: String,
        /// GitHub repository name
        repo: String,
        /// Column ID to treat as "done" (maps to closed issues)
        #[arg(long)]
        done_col: String,
    },
    /// Unlink a board from GitHub
    Unlink { board_id: String },
    /// Run a bidirectional sync for a board
    Sync { board_id: String },
}

#[derive(clap::Subcommand, Debug)]
enum LinearCommands {
    /// Save a Linear API key. Reads from stdin if not given.
    Connect {
        token: Option<String>,
    },
    /// Show token status and list accessible teams
    Status,
    /// List teams accessible with the saved token
    Teams,
    /// List projects for a team
    Projects {
        team_id: String,
    },
    /// Link a board to a Linear project (creates Monotask columns from Linear workflow states)
    Link {
        board_id: String,
        /// Linear team ID
        #[arg(long)]
        team: String,
        /// Linear project ID
        #[arg(long)]
        project: String,
        /// Optional: Monotask column ID to use as the done/completed column
        #[arg(long)]
        done_col: Option<String>,
    },
    /// Unlink a board from Linear
    Unlink { board_id: String },
    /// Run a bidirectional sync for a board
    Sync { board_id: String },
}

#[derive(clap::Subcommand, Debug)]
enum MailCommands {
    /// Connect Gmail via OAuth2 PKCE (BYO Google Cloud client ID)
    GmailConnect {
        /// Google OAuth2 client ID from Google Cloud Console
        #[arg(long)]
        client_id: String,
    },
    /// Connect Outlook via OAuth2 PKCE (BYO Azure client ID)
    OutlookConnect {
        /// Microsoft OAuth2 client ID from Azure portal
        #[arg(long)]
        client_id: String,
        /// Azure tenant ID, or "common" for personal+work accounts
        #[arg(long, default_value = "common")]
        tenant_id: String,
    },
    /// Show connection status for all providers
    Status,
    /// Remove saved credentials for a provider
    Disconnect {
        /// Provider: gmail | outlook
        provider: String,
    },
    /// Link a board to receive email contacts
    Link {
        board_id: String,
        /// Provider(s) to sync: gmail | outlook | imap | both
        #[arg(long, default_value = "both")]
        provider: String,
        /// Google OAuth2 Client ID (required for Gmail)
        #[arg(long)]
        gmail_client_id: Option<String>,
        /// Azure OAuth2 Client ID (required for Outlook)
        #[arg(long)]
        outlook_client_id: Option<String>,
        /// Azure tenant ID (default: common)
        #[arg(long)]
        tenant_id: Option<String>,
        /// Column ID for new contacts (defaults to first column)
        #[arg(long)]
        inbox_col: Option<String>,
        /// Number of recent emails to keep per contact as comments (default 2)
        #[arg(long, default_value = "2")]
        keep_last: u64,
    },
    /// Connect via IMAP (username + password — works with any provider)
    ImapConnect {
        /// IMAP server hostname (e.g. imap.gmail.com)
        #[arg(long)]
        host: String,
        /// IMAP port (default 993 for TLS)
        #[arg(long, default_value = "993")]
        port: u16,
        /// Email address / username
        #[arg(long)]
        username: String,
        /// Password or app-specific password
        #[arg(long)]
        password: Option<String>,
        /// Mailbox folder to sync (default INBOX)
        #[arg(long, default_value = "INBOX")]
        folder: String,
    },
    /// Show IMAP credential status and test the connection
    ImapStatus,
    /// Remove saved IMAP credentials
    ImapDisconnect,
    /// Unlink a board from email sync
    Unlink { board_id: String },
    /// Sync emails into a board
    Sync { board_id: String },
}

#[derive(Subcommand)]
enum FieldCommands {
    /// Create a new custom field definition on a board
    Create {
        board_id: String,
        /// Human-readable field name (must be unique within the board)
        name: String,
        /// Type: text, number, date, select, multi_select, checkbox
        #[arg(long, default_value = "text")]
        field_type: String,
        /// Allowed option (repeat for each choice; required for select/multi_select)
        #[arg(long = "option")]
        options: Vec<String>,
        /// Default value written to new cards when --auto-apply is set
        #[arg(long)]
        default_value: Option<String>,
        /// Write default to every new card automatically at creation time
        #[arg(long)]
        auto_apply: bool,
        #[arg(long)] json: bool,
    },
    /// List all (non-archived) field definitions on a board
    List { board_id: String, #[arg(long)] json: bool },
    /// Rename a field (by name or UUID)
    Rename {
        board_id: String,
        /// Field UUID or name
        field_ref: String,
        new_name: String,
        #[arg(long)] json: bool,
    },
    /// Archive (soft-delete) a field definition
    Delete {
        board_id: String,
        /// Field UUID or name
        field_ref: String,
        #[arg(long)] json: bool,
    },
    /// Apply the field's default value to all cards that do not have it set yet
    Backfill {
        board_id: String,
        /// Field UUID or name
        field_ref: String,
        #[arg(long)] json: bool,
    },
    /// Update a field's default_value and/or auto_apply flag
    Update {
        board_id: String,
        /// Field UUID or name
        field_ref: String,
        #[arg(long)]
        default_value: Option<String>,
        #[arg(long)]
        auto_apply: Option<bool>,
        #[arg(long)] json: bool,
    },
}

#[derive(Subcommand)]
enum AppCommands {
    /// Open a monotask:// URL in the desktop app (board or card deep link)
    Open {
        /// URL to open, e.g. monotask://board/<id> or monotask://board/<id>/card/<id>
        url: String,
    },
}

#[derive(Subcommand)]
enum ChatCommands {
    /// Send a chat message to a space
    Send {
        space_id: String,
        /// Message text
        text: String,
        #[arg(long)] json: bool,
    },
    /// List recent chat messages in a space
    List {
        space_id: String,
        /// Maximum number of messages to return (default: 50)
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)] json: bool,
    },
}

fn data_dir(cli: &Cli) -> anyhow::Result<std::path::PathBuf> {
    if let Some(d) = &cli.data_dir {
        return Ok(d.clone());
    }
    let base = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!(
            "Cannot determine data directory. Use --data-dir to specify one explicitly."
        ))?;
    // Migrate data from old "p2p-kanban" directory if new "monotask" dir doesn't exist yet
    let new_dir = base.join("monotask");
    let old_dir = base.join("p2p-kanban");
    if !new_dir.exists() && old_dir.exists() {
        let _ = std::fs::rename(&old_dir, &new_dir);
    }
    Ok(new_dir)
}

fn load_cli_identity(data_dir: &std::path::Path, conn: &rusqlite::Connection) -> anyhow::Result<monotask_crypto::Identity> {
    use monotask_crypto::Identity;
    use monotask_storage::space as space_store;
    let key_path = data_dir.join("identity.key");
    // Step 1: Try SSH key from profile
    if let Some(profile) = space_store::get_profile(conn)? {
        if let Some(ssh_path) = &profile.ssh_key_path {
            let p = std::path::Path::new(ssh_path);
            if p.exists() {
                if let Ok(id) = monotask_crypto::import_ssh_identity(Some(p)) {
                    return Ok(id);
                }
            }
        }
    }
    // Step 2: Fall back to identity.key
    if key_path.exists() {
        let bytes = std::fs::read(&key_path)?;
        if bytes.len() == 32 {
            let arr: [u8; 32] = bytes.try_into().map_err(|_| anyhow::anyhow!("bad key len"))?;
            return Ok(Identity::from_secret_bytes(&arr));
        }
    }
    // Step 3: Generate new identity
    let id = Identity::generate();
    std::fs::write(&key_path, id.to_secret_bytes())?;
    let new_profile = monotask_core::space::UserProfile {
        pubkey: id.public_key_hex(),
        display_name: None,
        avatar_blob: None,
        bio: None,
        role: None,
        color_accent: None,
        presence: None,
        ssh_key_path: None,
    };
    space_store::upsert_profile(conn, &new_profile)?;
    Ok(id)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let dir = data_dir(&cli)?;
    let mut storage = monotask_storage::Storage::open(&dir)?;
    let identity = load_cli_identity(&dir, storage.conn())?;

    match cli.command {
        Commands::Init => {
            println!("Initialized monotaskcli at {}", dir.display());
        }
        Commands::Board { cmd } => match cmd {
            BoardCommands::Create { title, space, json } => {
                use monotask_core::space as cs;
                use monotask_storage::space as ss;
                let id = monotask_crypto::Identity::generate();
                let (mut doc, board) = monotask_core::board::create_board(&title, &id.public_key_hex())?;
                storage.save_board(&board.id, &mut doc)?;
                let space_bytes = ss::load_space_doc(storage.conn(), &space)
                    .map_err(|_| anyhow::anyhow!("Space '{}' not found. Run `monotaskcli space list` to see available spaces.", space))?;
                let mut space_doc = automerge::AutoCommit::load(&space_bytes)?;
                cs::add_board_ref(&mut space_doc, &board.id)?;
                ss::update_space_doc(storage.conn(), &space, &space_doc.save())?;
                ss::add_board(storage.conn(), &space, &board.id)?;
                if json {
                    let deep_link = format!("monotask://board/{}", board.id);
                    println!("{}", serde_json::json!({"id": board.id, "title": board.title, "space_id": space, "deep_link": deep_link}));
                } else {
                    println!("Created board: {} ({}) in space {}", board.title, board.id, space);
                }
            }
            BoardCommands::List { json } => {
                let ids = storage.list_board_ids()?;
                if json {
                    let boards: Vec<serde_json::Value> = ids.iter().map(|id| {
                        let title = storage.load_board(id)
                            .ok()
                            .and_then(|doc| monotask_core::board::get_board_title(&doc).ok())
                            .unwrap_or_default();
                        serde_json::json!({"id": id, "title": title})
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&boards)?);
                } else {
                    for id in &ids {
                        let title = storage.load_board(id)
                            .ok()
                            .and_then(|doc| monotask_core::board::get_board_title(&doc).ok())
                            .unwrap_or_default();
                        println!("{id}: {title}");
                    }
                }
            }
            BoardCommands::Rename { board_id, new_title, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::rename_board(&mut doc, &new_title)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"board_id": board_id, "title": new_title})); }
                else { println!("Renamed board {} to: {}", board_id, new_title); }
            }
            BoardCommands::Delete { board_id, space, json } => {
                use monotask_core::space as cs;
                use monotask_storage::space as ss;
                let space_bytes = ss::load_space_doc(storage.conn(), &space)
                    .map_err(|_| anyhow::anyhow!("Space '{}' not found. Run `monotaskcli space list` to see available spaces.", space))?;
                let mut space_doc = automerge::AutoCommit::load(&space_bytes)?;
                cs::remove_board_ref(&mut space_doc, &board_id)?;
                ss::update_space_doc(storage.conn(), &space, &space_doc.save())?;
                ss::remove_board(storage.conn(), &space, &board_id)?;
                storage.delete_board(&board_id)?;
                if json { println!("{}", serde_json::json!({"deleted": true, "board_id": board_id, "space_id": space})); }
                else { println!("Deleted board {} from space {}", board_id, space); }
            }
            BoardCommands::Undo { board_id, json } => {
                let actor_key = identity.public_key_hex();
                let conn = storage.conn();
                let row: Option<(i64, String, Vec<u8>)> = conn.query_row(
                    "SELECT seq, action_tag, inverse_op FROM undo_stack WHERE board_id = ?1 AND actor_key = ?2 ORDER BY seq DESC LIMIT 1",
                    rusqlite::params![board_id, actor_key],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                ).ok();
                let (seq, action_tag, prev_bytes) = match row {
                    None => {
                        if json { println!("{}", serde_json::json!({"ok": false, "reason": "nothing to undo"})); }
                        else { println!("Nothing to undo."); }
                        return Ok(());
                    }
                    Some(r) => r,
                };
                let mut cur = monotask_storage::board::load_board(conn, &board_id)
                    .map_err(|e| anyhow::anyhow!(e))?;
                let cur_bytes = cur.save();
                let redo_seq: i64 = conn.query_row(
                    "SELECT COALESCE(MAX(seq), 0) + 1 FROM redo_stack WHERE board_id = ?1 AND actor_key = ?2",
                    rusqlite::params![board_id, actor_key], |r| r.get(0)).unwrap_or(1);
                let hlc = monotask_core::clock::now();
                conn.execute(
                    "INSERT INTO redo_stack (board_id, actor_key, seq, action_tag, forward_op, hlc) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![board_id, actor_key, redo_seq, action_tag, &cur_bytes, hlc],
                )?;
                let mut prev_doc = automerge::AutoCommit::load(&prev_bytes)?;
                monotask_storage::board::save_board(conn, &board_id, &mut prev_doc)
                    .map_err(|e| anyhow::anyhow!(e))?;
                conn.execute(
                    "DELETE FROM undo_stack WHERE board_id = ?1 AND actor_key = ?2 AND seq = ?3",
                    rusqlite::params![board_id, actor_key, seq],
                )?;
                if json { println!("{}", serde_json::json!({"ok": true})); }
                else { println!("Undo successful."); }
            }
            BoardCommands::Redo { board_id, json } => {
                let actor_key = identity.public_key_hex();
                let conn = storage.conn();
                let row: Option<(i64, String, Vec<u8>)> = conn.query_row(
                    "SELECT seq, action_tag, forward_op FROM redo_stack WHERE board_id = ?1 AND actor_key = ?2 ORDER BY seq DESC LIMIT 1",
                    rusqlite::params![board_id, actor_key],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                ).ok();
                let (seq, action_tag, forward_bytes) = match row {
                    None => {
                        if json { println!("{}", serde_json::json!({"ok": false, "reason": "nothing to redo"})); }
                        else { println!("Nothing to redo."); }
                        return Ok(());
                    }
                    Some(r) => r,
                };
                let mut cur = monotask_storage::board::load_board(conn, &board_id)
                    .map_err(|e| anyhow::anyhow!(e))?;
                let cur_bytes = cur.save();
                let undo_seq: i64 = conn.query_row(
                    "SELECT COALESCE(MAX(seq), 0) + 1 FROM undo_stack WHERE board_id = ?1 AND actor_key = ?2",
                    rusqlite::params![board_id, actor_key], |r| r.get(0)).unwrap_or(1);
                let hlc = monotask_core::clock::now();
                conn.execute(
                    "INSERT INTO undo_stack (board_id, actor_key, seq, action_tag, inverse_op, hlc) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![board_id, actor_key, undo_seq, action_tag, &cur_bytes, hlc],
                )?;
                let mut fwd_doc = automerge::AutoCommit::load(&forward_bytes)?;
                monotask_storage::board::save_board(conn, &board_id, &mut fwd_doc)
                    .map_err(|e| anyhow::anyhow!(e))?;
                conn.execute(
                    "DELETE FROM redo_stack WHERE board_id = ?1 AND actor_key = ?2 AND seq = ?3",
                    rusqlite::params![board_id, actor_key, seq],
                )?;
                if json { println!("{}", serde_json::json!({"ok": true})); }
                else { println!("Redo successful."); }
            }
            BoardCommands::Schema { board_id, json } => {
                let doc = storage.load_board(&board_id)?;
                let title = monotask_core::board::get_board_title(&doc).unwrap_or_default();
                let cols = monotask_core::column::list_columns(&doc)?;
                let fields = monotask_core::field::list_fields(&doc)?;
                if json {
                    let cols_json: Vec<serde_json::Value> = cols.iter()
                        .map(|c| serde_json::json!({"id": c.id, "title": c.title}))
                        .collect();
                    let fields_json: Vec<serde_json::Value> = fields.iter()
                        .map(|f| serde_json::json!({
                            "id": f.id, "name": f.name, "type": f.field_type.as_str(),
                            "options": f.options, "default_value": f.default_value,
                            "auto_apply": f.auto_apply, "archived": f.archived,
                        }))
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                        "board_id": board_id, "title": title,
                        "columns": cols_json, "fields": fields_json,
                    }))?);
                } else {
                    println!("Board: {} ({})", title, board_id);
                    println!("\nColumns ({}):", cols.len());
                    for c in &cols { println!("  {} — {}", c.id, c.title); }
                    println!("\nFields ({}):", fields.len());
                    if fields.is_empty() { println!("  (none)"); }
                    for f in &fields {
                        let dv = f.default_value.as_deref().map(|v| format!(" default={v}")).unwrap_or_default();
                        let aa = if f.auto_apply { " auto_apply" } else { "" };
                        println!("  {} — {} [{}]{}{}", f.id, f.name, f.field_type.as_str(), dv, aa);
                    }
                }
            }
        },
        Commands::Column { cmd } => match cmd {
            ColumnCommands::Create { board_id, title, json } => {
                let mut doc = storage.load_board(&board_id)?;
                let col_id = monotask_core::column::create_column(&mut doc, &title)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"id": col_id, "board_id": board_id})); }
                else { println!("Created column: {title} ({col_id})"); }
            }
            ColumnCommands::List { board_id, json } => {
                let doc = storage.load_board(&board_id)?;
                let cols = monotask_core::column::list_columns(&doc)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&cols)?);
                } else {
                    for col in &cols {
                        println!("{}: {}", col.id, col.title);
                    }
                }
            }
            ColumnCommands::Rename { board_id, col_id, new_title, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::rename_column_by_id(&mut doc, &col_id, &new_title)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"col_id": col_id, "title": new_title})); }
                else { println!("Renamed column {} to: {}", col_id, new_title); }
            }
            ColumnCommands::Delete { board_id, col_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::delete_column(&mut doc, &col_id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"deleted": col_id})); }
                else { println!("Deleted column {col_id}"); }
            }
        },
        Commands::Card { cmd } => match cmd {
            CardCommands::Create { board_id, col_id, title, fields, json } => {
                let mut doc = storage.load_board(&board_id)?;
                // Parse and validate all --field pairs BEFORE creating the card
                let field_pairs = parse_field_assignments(&fields)?;
                let mut resolved_fields: Vec<(monotask_core::field::FieldDefinition, String)> = Vec::new();
                for (key, value) in field_pairs {
                    let def = monotask_core::field::resolve_field_ref(&doc, &key)
                        .map_err(|e| anyhow::anyhow!("{e}"))?
                        .ok_or_else(|| anyhow::anyhow!("field '{}' not found on this board", key))?;
                    monotask_core::field::validate_field_value(&def, &value)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    resolved_fields.push((def, value));
                }
                let actor_pk = vec![0u8; 32];
                let members = vec![actor_pk.clone()];
                let card = monotask_core::card::create_card(&mut doc, &col_id, &title, &actor_pk, &members)?;
                // Write explicit fields first, then apply auto-apply defaults (explicit beats default)
                for (def, value) in &resolved_fields {
                    monotask_core::field::set_card_field(&mut doc, &card.id, &def.id, value)?;
                }
                monotask_core::field::apply_default_fields(&mut doc, &card.id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    let number_display = card.number.as_ref().map(|n| n.to_display());
                    println!("{}", serde_json::json!({"id": card.id, "title": card.title, "board_id": board_id, "number": number_display}));
                } else {
                    println!("Created card: {} ({})", card.title, card.id);
                }
            }
            CardCommands::List { board_id, col: col_filter, label: label_filter, filters, json } => {
                use automerge::ReadDoc;
                let doc = storage.load_board(&board_id)?;

                // Parse --where expressions and resolve field IDs up front
                let mut field_filters: Vec<(String, monotask_core::field::FieldType, String, String)> = Vec::new();
                for expr in &filters {
                    let (field_ref, op, value) = parse_filter_expr(expr)
                        .ok_or_else(|| anyhow::anyhow!("invalid --where expression '{}'. Use FIELD=VALUE, FIELD>VALUE, FIELD~SUBSTRING etc.", expr))?;
                    let def = monotask_core::field::resolve_field_ref(&doc, &field_ref)
                        .map_err(|e| anyhow::anyhow!("{e}"))?
                        .ok_or_else(|| anyhow::anyhow!("field '{}' not found on this board", field_ref))?;
                    field_filters.push((def.id, def.field_type, op, value));
                }

                // Build set of matching card IDs from field filters (AND semantics)
                let filter_card_ids: Option<std::collections::HashSet<String>> = if field_filters.is_empty() {
                    None
                } else {
                    let mut sets: Vec<std::collections::HashSet<String>> = Vec::new();
                    for (field_id, field_type, op, value) in &field_filters {
                        let ids = storage.query_cards_by_field(&board_id, field_id, field_type, op, value)?;
                        sets.push(ids.into_iter().collect());
                    }
                    // Intersect all sets
                    let intersection = sets.into_iter().reduce(|a, b| a.intersection(&b).cloned().collect());
                    intersection
                };

                let cols = monotask_core::column::list_columns(&doc)?;
                let mut cards: Vec<(String, String, monotask_core::card::Card)> = Vec::new();
                for col in &cols {
                    if let Some(ref cf) = col_filter {
                        if &col.id != cf { continue; }
                    }
                    let col_obj = match monotask_core::column::find_column_obj(&doc, &col.id)? {
                        Some(o) => o,
                        None => continue,
                    };
                    let card_ids_list = match monotask_core::column::get_card_ids_list(&doc, &col_obj) {
                        Ok(id) => id,
                        Err(_) => continue,
                    };
                    for i in 0..doc.length(&card_ids_list) {
                        if let Some((automerge::Value::Scalar(s), _)) = doc.get(&card_ids_list, i)? {
                            if let automerge::ScalarValue::Str(card_id) = s.as_ref() {
                                if let Some(ref allowed) = filter_card_ids {
                                    if !allowed.contains(card_id.as_str()) { continue; }
                                }
                                if let Ok(card) = monotask_core::card::read_card(&doc, card_id.as_str()) {
                                    if card.deleted || card.archived { continue; }
                                    if let Some(ref lf) = label_filter {
                                        if !card.labels.iter().any(|l| l == lf) { continue; }
                                    }
                                    cards.push((col.id.clone(), col.title.clone(), card));
                                }
                            }
                        }
                    }
                }
                if json {
                    let out: Vec<serde_json::Value> = cards.iter().map(|(col_id, col_title, card)| {
                        let mut v = serde_json::to_value(card).unwrap_or_default();
                        v["col_id"] = serde_json::json!(col_id);
                        v["col_title"] = serde_json::json!(col_title);
                        v
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&out)?);
                } else {
                    for (_, col_title, card) in &cards {
                        let num = card.number.as_ref().map(|n| n.to_display()).unwrap_or_default();
                        println!("[{}] {} – {} ({})", col_title, num, card.title, card.id);
                    }
                }
            }
            CardCommands::View { board_id, card_id, json } => {
                let doc = storage.load_board(&board_id)?;
                let card = monotask_core::card::read_card(&doc, &card_id)?;
                let parent_ref = monotask_core::card::get_parent_ref(&doc, &card_id).unwrap_or(None);
                let subtask_refs = monotask_core::card::list_subtask_refs(&doc, &card_id).unwrap_or_default();
                if json {
                    let parent_json = parent_ref.as_ref().map(|(bid, cid)| serde_json::json!({"board_id": bid, "card_id": cid}));
                    let subtasks_json: Vec<_> = subtask_refs.iter().map(|(bid, cid)| serde_json::json!({"board_id": bid, "card_id": cid})).collect();
                    let mut v = serde_json::to_value(&card)?;
                    v["parent"] = serde_json::to_value(parent_json)?;
                    v["subtasks"] = serde_json::to_value(subtasks_json)?;
                    println!("{}", serde_json::to_string_pretty(&v)?);
                } else {
                    println!("ID:          {}", card.id);
                    println!("Title:       {}", card.title);
                    if !card.description.is_empty() {
                        println!("Description: {}", card.description);
                    }
                    if card.deleted { println!("Status:      DELETED"); }
                    else if card.archived { println!("Status:      archived"); }
                    if let Some(due) = &card.due_date { println!("Due:         {due}"); }
                    if card.impact.is_some() || card.effort.is_some() {
                        let imp = card.impact.unwrap_or(0);
                        let eff = card.effort.unwrap_or(0);
                        let pri = monotask_core::card::compute_priority(imp, eff);
                        println!("Impact:      {imp}/10");
                        println!("Effort:      {eff}/10");
                        println!("Priority:    {pri}/10");
                    } else if let Some(dp) = card.direct_priority {
                        println!("Priority:    {dp}/10");
                    }
                    if let Some((pbid, pcid)) = &parent_ref {
                        println!("Parent:      {} (board: {})", pcid, pbid);
                    }
                    if !subtask_refs.is_empty() {
                        println!("Subtasks ({}):", subtask_refs.len());
                        for (bid, cid) in &subtask_refs {
                            println!("  {} (board: {})", cid, bid);
                        }
                    }
                }
            }
            CardCommands::Rename { board_id, card_id, new_title, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::rename_card(&mut doc, &card_id, &new_title)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "title": new_title})); }
                else { println!("Renamed card {} to: {}", card_id, new_title); }
            }
            CardCommands::Delete { board_id, card_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::delete_card(&mut doc, &card_id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"deleted": card_id})); }
                else { println!("Deleted card {card_id}"); }
            }
            CardCommands::Archive { board_id, card_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::archive_card(&mut doc, &card_id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"archived": card_id})); }
                else { println!("Archived card {card_id}"); }
            }
            CardCommands::Copy { board_id, card_id, col_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                let actor_pk = identity.to_secret_bytes().to_vec();
                let members = vec![actor_pk.clone()];
                let new_card = monotask_core::card::copy_card(&mut doc, &card_id, &col_id, &actor_pk, &members)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"id": new_card.id, "title": new_card.title})); }
                else { println!("Copied card to: {} ({})", new_card.title, new_card.id); }
            }
            CardCommands::Move { board_id, card_id, to_col_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                // Find which column currently contains the card
                let cols = monotask_core::column::list_columns(&doc)?;
                let from_col_id = {
                    use automerge::ReadDoc;
                    let mut found = None;
                    'outer: for col in &cols {
                        let col_obj = match monotask_core::column::find_column_obj(&doc, &col.id)? {
                            Some(o) => o,
                            None => continue,
                        };
                        let card_ids = match monotask_core::column::get_card_ids_list(&doc, &col_obj) {
                            Ok(id) => id,
                            Err(_) => continue,
                        };
                        for i in 0..doc.length(&card_ids) {
                            if let Some((automerge::Value::Scalar(s), _)) = doc.get(&card_ids, i)? {
                                if let automerge::ScalarValue::Str(text) = s.as_ref() {
                                    if text.as_str() == card_id {
                                        found = Some(col.id.clone());
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }
                    found.ok_or_else(|| anyhow::anyhow!("card {} not found in any column", card_id))?
                };
                monotask_core::column::move_card(&mut doc, &card_id, &from_col_id, &to_col_id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "to_col_id": to_col_id})); }
                else { println!("Moved card {} to column {}", card_id, to_col_id); }
            }
            CardCommands::SetDescription { board_id, card_id, text, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::set_description(&mut doc, &card_id, &text)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "description": text})); }
                else { println!("Updated description for card {card_id}"); }
            }
            CardCommands::SetCover { board_id, card_id, color, json } => {
                let color_arg = if color == "none" { "" } else { &color };
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::set_cover_color(&mut doc, &card_id, color_arg)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "color": color_arg})); }
                else { println!("Set cover color for card {card_id}"); }
            }
            CardCommands::SetDueDate { board_id, card_id, date, json } => {
                let due: Option<&str> = if date == "none" { None } else { Some(&date) };
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::set_due_date(&mut doc, &card_id, due)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "due_date": due})); }
                else { println!("Set due date for card {card_id}"); }
            }
            CardCommands::SetPriority { board_id, card_id, priority, json } => {
                let pri = if priority == "none" { "" } else { &priority };
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::set_priority(&mut doc, &card_id, pri)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "priority": pri})); }
                else { println!("Set priority for card {card_id}"); }
            }
            CardCommands::SetImpact { board_id, card_id, value, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::set_impact(&mut doc, &card_id, value)?;
                let card = monotask_core::card::read_card(&doc, &card_id)?;
                let effort = card.effort.unwrap_or(0);
                let priority = monotask_core::card::compute_priority(value, effort);
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "impact": value, "effort": effort, "priority": priority})); }
                else { println!("Impact={value}, Effort={effort} → Priority={priority}"); }
            }
            CardCommands::SetEffort { board_id, card_id, value, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::set_effort(&mut doc, &card_id, value)?;
                let card = monotask_core::card::read_card(&doc, &card_id)?;
                let impact = card.impact.unwrap_or(0);
                let priority = monotask_core::card::compute_priority(impact, value);
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "impact": impact, "effort": value, "priority": priority})); }
                else { println!("Impact={impact}, Effort={value} → Priority={priority}"); }
            }
            CardCommands::SetDirectPriority { board_id, card_id, value, clear, json } => {
                let v = if clear { None } else { value };
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::set_direct_priority(&mut doc, &card_id, v)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "direct_priority": v})); }
                else if let Some(p) = v { println!("Priority={p}/10"); }
                else { println!("Priority cleared"); }
            }
            CardCommands::ClearPriority { board_id, card_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::clear_priority_fields(&mut doc, &card_id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "cleared": true})); }
                else { println!("Impact, effort and priority cleared for card {card_id}"); }
            }
            CardCommands::SetAssignee { board_id, card_id, pubkey, json } => {
                let pk = if pubkey == "none" { "" } else { &pubkey };
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::set_assignee(&mut doc, &card_id, pk)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "assignee": pk})); }
                else { println!("Set assignee for card {card_id}"); }
            }
            CardCommands::AttachImage { board_id, card_id, file, json } => {
                use std::io::Read;
                use base64::Engine;
                let mut f = std::fs::File::open(&file)
                    .map_err(|e| anyhow::anyhow!("Cannot open {file}: {e}"))?;
                let mut bytes = Vec::new();
                f.read_to_end(&mut bytes)?;
                let data_b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let mime = mime_from_ext(&file);
                let name = std::path::Path::new(&file)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let id_raw = uuid::Uuid::new_v4().to_string().replace('-', "");
                let id = &id_raw[..6];
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::card::attach_image(&mut doc, &card_id, id, &name, mime, &data_b64)?;
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    println!("{}", serde_json::json!({"id": id, "name": name, "mime": mime, "token": format!("img:{id}")}));
                } else {
                    println!("Attached {} as img:{} — embed with ![{}](img:{})", name, id, name, id);
                }
            }
            CardCommands::ListAttachments { board_id, card_id, json } => {
                let doc = storage.load_board(&board_id)?;
                let card = monotask_core::card::read_card(&doc, &card_id)?;
                if json {
                    let atts: Vec<serde_json::Value> = card.attachments.iter()
                        .map(|(id, a)| serde_json::json!({"id": id, "name": a.name, "mime": a.mime, "size_b64": a.data_b64.len()}))
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&atts)?);
                } else if card.attachments.is_empty() {
                    println!("No attachments");
                } else {
                    for (id, a) in &card.attachments {
                        let kb = a.data_b64.len() * 3 / 4 / 1024;
                        println!("  img:{id}  {name}  ({mime}, ~{kb}KB)", name = a.name, mime = a.mime);
                    }
                }
            }
            CardCommands::SaveAttachment { board_id, card_id, attachment_id, output, json } => {
                use base64::Engine;
                let doc = storage.load_board(&board_id)?;
                let card = monotask_core::card::read_card(&doc, &card_id)?;
                let att = card.attachments.get(&attachment_id)
                    .ok_or_else(|| anyhow::anyhow!("Attachment {} not found", attachment_id))?;
                let bytes = base64::engine::general_purpose::STANDARD.decode(&att.data_b64)?;
                let out_path = output.unwrap_or_else(|| att.name.clone());
                std::fs::write(&out_path, &bytes)?;
                if json {
                    println!("{}", serde_json::json!({"saved": out_path, "size": bytes.len()}));
                } else {
                    println!("Saved {} ({} bytes) to {}", att.name, bytes.len(), out_path);
                }
            }
            CardCommands::Label { cmd } => match cmd {
                LabelCommands::Add { board_id, card_id, label, json } => {
                    let mut doc = storage.load_board(&board_id)?;
                    monotask_core::card::add_label(&mut doc, &card_id, &label)?;
                    storage.save_board(&board_id, &mut doc)?;
                    if json { println!("{}", serde_json::json!({"card_id": card_id, "label": label})); }
                    else { println!("Added label '{}' to card {}", label, card_id); }
                }
                LabelCommands::Remove { board_id, card_id, label, json } => {
                    let mut doc = storage.load_board(&board_id)?;
                    monotask_core::card::remove_label(&mut doc, &card_id, &label)?;
                    storage.save_board(&board_id, &mut doc)?;
                    if json { println!("{}", serde_json::json!({"card_id": card_id, "removed_label": label})); }
                    else { println!("Removed label '{}' from card {}", label, card_id); }
                }
                LabelCommands::List { board_id, card_id, json } => {
                    let doc = storage.load_board(&board_id)?;
                    let card = monotask_core::card::read_card(&doc, &card_id)?;
                    if json { println!("{}", serde_json::to_string_pretty(&card.labels)?); }
                    else {
                        if card.labels.is_empty() { println!("No labels."); }
                        else { for l in &card.labels { println!("{l}"); } }
                    }
                }
            },
            CardCommands::Subtask { cmd } => match cmd {
                SubtaskCommands::Add { parent_board_id, parent_card_id, child_board_id, col_id, title, json } => {
                    let actor_pk = vec![0u8; 32];
                    let members = vec![actor_pk.clone()];
                    if parent_board_id == child_board_id {
                        let mut doc = storage.load_board(&child_board_id)?;
                        let card = monotask_core::card::create_card(&mut doc, &col_id, &title, &actor_pk, &members)?;
                        monotask_core::card::set_parent_ref(&mut doc, &card.id, &parent_board_id, &parent_card_id)?;
                        monotask_core::card::add_subtask_ref(&mut doc, &parent_card_id, &child_board_id, &card.id)?;
                        storage.save_board(&child_board_id, &mut doc)?;
                        if json { println!("{}", serde_json::json!({"id": card.id, "title": card.title, "board_id": child_board_id})); }
                        else { println!("Created subtask: {} ({}) in board {}", card.title, card.id, child_board_id); }
                    } else {
                        let mut child_doc = storage.load_board(&child_board_id)?;
                        let card = monotask_core::card::create_card(&mut child_doc, &col_id, &title, &actor_pk, &members)?;
                        monotask_core::card::set_parent_ref(&mut child_doc, &card.id, &parent_board_id, &parent_card_id)?;
                        storage.save_board(&child_board_id, &mut child_doc)?;
                        let mut parent_doc = storage.load_board(&parent_board_id)?;
                        monotask_core::card::add_subtask_ref(&mut parent_doc, &parent_card_id, &child_board_id, &card.id)?;
                        storage.save_board(&parent_board_id, &mut parent_doc)?;
                        if json { println!("{}", serde_json::json!({"id": card.id, "title": card.title, "board_id": child_board_id})); }
                        else { println!("Created subtask: {} ({}) in board {}", card.title, card.id, child_board_id); }
                    }
                }
                SubtaskCommands::List { board_id, card_id, json } => {
                    let doc = storage.load_board(&board_id)?;
                    let refs = monotask_core::card::list_subtask_refs(&doc, &card_id)?;
                    if json {
                        let out: Vec<_> = refs.iter().map(|(bid, cid)| serde_json::json!({"board_id": bid, "card_id": cid})).collect();
                        println!("{}", serde_json::to_string_pretty(&out)?);
                    } else {
                        if refs.is_empty() { println!("No subtasks."); }
                        else {
                            for (bid, cid) in &refs {
                                println!("{} (board: {})", cid, bid);
                            }
                        }
                    }
                }
            },
            CardCommands::FieldSet { board_id, card_id, field_ref, value, json } => {
                let card_id = storage.resolve_card_ref(&board_id, &card_id)?;
                let mut doc = storage.load_board(&board_id)?;
                let def = monotask_core::field::resolve_field_ref(&doc, &field_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .ok_or_else(|| anyhow::anyhow!("field '{}' not found on this board", field_ref))?;
                monotask_core::field::validate_field_value(&def, &value)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                monotask_core::field::set_card_field(&mut doc, &card_id, &def.id, &value)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"card_id": card_id, "field_id": def.id, "field_name": def.name, "value": value})); }
                else { println!("Set {}: {} on card {}", def.name, value, card_id); }
            }
            CardCommands::FieldGet { board_id, card_id, field_ref, json } => {
                let card_id = storage.resolve_card_ref(&board_id, &card_id)?;
                let doc = storage.load_board(&board_id)?;
                let def = monotask_core::field::resolve_field_ref(&doc, &field_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .ok_or_else(|| anyhow::anyhow!("field '{}' not found on this board", field_ref))?;
                let val = monotask_core::field::get_card_field(&doc, &card_id, &def.id)?;
                if json {
                    println!("{}", serde_json::json!({"field_id": def.id, "field_name": def.name, "value": val}));
                } else {
                    match val {
                        Some(v) => println!("{}: {}", def.name, v),
                        None => println!("{}: (not set)", def.name),
                    }
                }
            }
            CardCommands::FieldClear { board_id, card_id, field_ref, json } => {
                let card_id = storage.resolve_card_ref(&board_id, &card_id)?;
                let mut doc = storage.load_board(&board_id)?;
                let def = monotask_core::field::resolve_field_ref(&doc, &field_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .ok_or_else(|| anyhow::anyhow!("field '{}' not found on this board", field_ref))?;
                monotask_core::field::clear_card_field(&mut doc, &card_id, &def.id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"cleared": def.id, "card_id": card_id})); }
                else { println!("Cleared {} from card {}", def.name, card_id); }
            }
            CardCommands::FieldList { board_id, card_id, json } => {
                let card_id = storage.resolve_card_ref(&board_id, &card_id)?;
                let doc = storage.load_board(&board_id)?;
                let pairs = monotask_core::field::list_card_fields(&doc, &card_id)?;
                // Resolve field IDs to names for display
                let mut out: Vec<serde_json::Value> = Vec::new();
                for (field_id, value) in pairs {
                    let name = monotask_core::field::get_field_by_id(&doc, &field_id)
                        .ok().flatten()
                        .map(|d| d.name)
                        .unwrap_or_else(|| field_id.clone());
                    out.push(serde_json::json!({"field_id": field_id, "name": name, "value": value}));
                }
                if json {
                    println!("{}", serde_json::to_string_pretty(&out)?);
                } else if out.is_empty() {
                    println!("No custom fields set.");
                } else {
                    for entry in &out {
                        println!("{}: {}", entry["name"].as_str().unwrap_or(""), entry["value"].as_str().unwrap_or(""));
                    }
                }
            }
            CardCommands::Upsert { board_id, col_id, title, match_field, match_value, fields, json } => {
                let mut doc = storage.load_board(&board_id)?;
                // Resolve match field
                let match_def = monotask_core::field::resolve_field_ref(&doc, &match_field)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .ok_or_else(|| anyhow::anyhow!("match field '{}' not found on this board", match_field))?;
                // Parse and validate all --field pairs
                let field_pairs = parse_field_assignments(&fields)?;
                let mut resolved_fields: Vec<(monotask_core::field::FieldDefinition, String)> = Vec::new();
                for (key, value) in field_pairs {
                    let def = monotask_core::field::resolve_field_ref(&doc, &key)
                        .map_err(|e| anyhow::anyhow!("{e}"))?
                        .ok_or_else(|| anyhow::anyhow!("field '{}' not found on this board", key))?;
                    monotask_core::field::validate_field_value(&def, &value)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    resolved_fields.push((def, value));
                }
                // Search existing non-deleted cards for match_field == match_value
                let cards_map = monotask_core::get_cards_map_readonly(&doc)?;
                use automerge::ReadDoc;
                let all_card_ids: Vec<String> = doc.keys(&cards_map).map(|k| k.to_string()).collect();
                let mut existing_card_id: Option<String> = None;
                for cid in all_card_ids {
                    let val = monotask_core::field::get_card_field(&doc, &cid, &match_def.id)?;
                    if val.as_deref() == Some(&match_value) {
                        // Verify not deleted
                        let card_obj = match doc.get(&cards_map, cid.as_str())? {
                            Some((_, o)) => o, None => continue,
                        };
                        let is_deleted = match doc.get(&card_obj, "deleted")? {
                            Some((automerge::Value::Scalar(s), _)) => matches!(s.as_ref(), automerge::ScalarValue::Boolean(true)),
                            _ => false,
                        };
                        if !is_deleted { existing_card_id = Some(cid); break; }
                    }
                }
                let (card_id, was_created) = if let Some(eid) = existing_card_id {
                    (eid, false)
                } else {
                    let actor_pk = vec![0u8; 32];
                    let members = vec![actor_pk.clone()];
                    let card = monotask_core::card::create_card(&mut doc, &col_id, &title, &actor_pk, &members)?;
                    // Set match field value on new card
                    monotask_core::field::set_card_field(&mut doc, &card.id, &match_def.id, &match_value)?;
                    (card.id, true)
                };
                for (def, value) in &resolved_fields {
                    monotask_core::field::set_card_field(&mut doc, &card_id, &def.id, value)?;
                }
                if was_created {
                    monotask_core::field::apply_default_fields(&mut doc, &card_id)?;
                }
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    println!("{}", serde_json::json!({"card_id": card_id, "created": was_created, "board_id": board_id}));
                } else if was_created {
                    println!("Created card {} ({})", title, card_id);
                } else {
                    println!("Updated card {}", card_id);
                }
            }
            CardCommands::Prerequisite { cmd } => match cmd {
                PrerequisiteCommands::Add { board_id, card_id, prereq_board_id, prereq_card_id, json } => {
                    if card_id == prereq_card_id && board_id == prereq_board_id {
                        return Err(anyhow::anyhow!("A card cannot be its own prerequisite"));
                    }
                    let mut doc = storage.load_board(&board_id)?;
                    monotask_core::card::add_prerequisite_ref(&mut doc, &card_id, &prereq_board_id, &prereq_card_id)?;
                    storage.save_board(&board_id, &mut doc)?;
                    if json { println!("{}", serde_json::json!({"board_id": board_id, "card_id": card_id, "prereq_board_id": prereq_board_id, "prereq_card_id": prereq_card_id})); }
                    else { println!("Added prerequisite {} (board: {}) to card {} (board: {})", prereq_card_id, prereq_board_id, card_id, board_id); }
                },
                PrerequisiteCommands::List { board_id, card_id, json } => {
                    let doc = storage.load_board(&board_id)?;
                    let refs = monotask_core::card::list_prerequisite_refs(&doc, &card_id)?;
                    if json {
                        let out: Vec<_> = refs.iter().map(|(bid, cid)| serde_json::json!({"board_id": bid, "card_id": cid})).collect();
                        println!("{}", serde_json::to_string_pretty(&out)?);
                    } else {
                        if refs.is_empty() { println!("No prerequisites."); }
                        else {
                            for (bid, cid) in &refs {
                                println!("{} (board: {})", cid, bid);
                            }
                        }
                    }
                },
                PrerequisiteCommands::Remove { board_id, card_id, prereq_board_id, prereq_card_id, json } => {
                    let mut doc = storage.load_board(&board_id)?;
                    monotask_core::card::remove_prerequisite_ref(&mut doc, &card_id, &prereq_board_id, &prereq_card_id)?;
                    storage.save_board(&board_id, &mut doc)?;
                    if json { println!("{}", serde_json::json!({"ok": true})); }
                    else { println!("Removed prerequisite {} (board: {}) from card {} (board: {})", prereq_card_id, prereq_board_id, card_id, board_id); }
                },
            },
            CardCommands::Link { cmd } => match cmd {
                LinkCommands::Add { board_id, card_id, target_board_id, target_card_id, json } => {
                    let mut doc = storage.load_board(&board_id)?;
                    monotask_core::card::add_card_link(&mut doc, &card_id, &target_board_id, &target_card_id)?;
                    storage.save_board(&board_id, &mut doc)?;
                    if json { println!("{}", serde_json::json!({"ok": true, "board_id": board_id, "card_id": card_id, "target_board_id": target_board_id, "target_card_id": target_card_id})); }
                    else { println!("Linked {card_id} → {target_card_id} (board: {target_board_id})"); }
                }
                LinkCommands::List { board_id, card_id, json } => {
                    let doc = storage.load_board(&board_id)?;
                    let links = monotask_core::card::list_card_links(&doc, &card_id)?;
                    if json {
                        let out: Vec<_> = links.iter().map(|(bid, cid)| serde_json::json!({"board_id": bid, "card_id": cid})).collect();
                        println!("{}", serde_json::to_string_pretty(&out)?);
                    } else if links.is_empty() {
                        println!("No links.");
                    } else {
                        for (bid, cid) in &links { println!("{} (board: {})", cid, bid); }
                    }
                }
                LinkCommands::Remove { board_id, card_id, target_board_id, target_card_id, json } => {
                    let mut doc = storage.load_board(&board_id)?;
                    monotask_core::card::remove_card_link(&mut doc, &card_id, &target_board_id, &target_card_id)?;
                    storage.save_board(&board_id, &mut doc)?;
                    if json { println!("{}", serde_json::json!({"ok": true})); }
                    else { println!("Removed link {card_id} → {target_card_id}"); }
                }
            },
            CardCommands::Mentions { board_id, card_id, json } => {
                let mut stmt = storage.conn().prepare(
                    "SELECT mention_token, created_at FROM mention_index \
                     WHERE board_id = ?1 AND card_id = ?2 ORDER BY created_at"
                ).map_err(|e| anyhow::anyhow!(e))?;
                let rows: Vec<(String, String)> = stmt.query_map(
                    rusqlite::params![board_id, card_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                ).map_err(|e| anyhow::anyhow!(e))?
                 .filter_map(|r| r.ok())
                 .collect();
                if json {
                    let out: Vec<_> = rows.iter()
                        .map(|(token, ts)| serde_json::json!({"mention": token, "created_at": ts}))
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&out)?);
                } else if rows.is_empty() {
                    println!("No mentions.");
                } else {
                    for (token, ts) in &rows { println!("@{} ({})", token, ts); }
                }
            },
            CardCommands::Comment { cmd } => match cmd {
                CommentCommands::Add { board_id, card_id, text, json } => {
                    let mut doc = storage.load_board(&board_id)?;
                    let author_key = identity.public_key_hex();
                    let comment = monotask_core::comment::add_comment(&mut doc, &card_id, &text, &author_key, None, None, None)?;
                    storage.save_board(&board_id, &mut doc)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&comment)?);
                    } else {
                        println!("Added comment {}", comment.id);
                    }
                }
                CommentCommands::List { board_id, card_id, json } => {
                    let doc = storage.load_board(&board_id)?;
                    let comments = monotask_core::comment::list_comments(&doc, &card_id)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&comments)?);
                    } else {
                        for c in &comments {
                            let image_tag = if c.image_b64.is_some() { " [+image]" } else { "" };
                            println!("[{}] {}: {}{}", c.created_at, c.author, c.text, image_tag);
                        }
                    }
                }
                CommentCommands::Delete { board_id, card_id, comment_id, json } => {
                    let mut doc = storage.load_board(&board_id)?;
                    monotask_core::comment::delete_comment(&mut doc, &card_id, &comment_id)?;
                    storage.save_board(&board_id, &mut doc)?;
                    if json {
                        println!("{}", serde_json::json!({"deleted": comment_id}));
                    } else {
                        println!("Deleted comment {comment_id}");
                    }
                }
                CommentCommands::Edit { board_id, card_id, comment_id, new_text, json } => {
                    let mut doc = storage.load_board(&board_id)?;
                    monotask_core::comment::edit_comment(&mut doc, &card_id, &comment_id, &new_text)?;
                    storage.save_board(&board_id, &mut doc)?;
                    if json {
                        println!("{}", serde_json::json!({"edited": comment_id}));
                    } else {
                        println!("Edited comment {comment_id}");
                    }
                }
            },
        },
        Commands::Checklist { cmd } => match cmd {
            ChecklistCommands::Add { board_id, card_id, title, json } => {
                let mut doc = storage.load_board(&board_id)?;
                let cl = monotask_core::checklist::add_checklist(&mut doc, &card_id, &title)?;
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&cl)?);
                } else {
                    println!("Created checklist: {} ({})", cl.title, cl.id);
                }
            }
            ChecklistCommands::ItemAdd { board_id, card_id, checklist_id, text, json } => {
                let mut doc = storage.load_board(&board_id)?;
                let item = monotask_core::checklist::add_checklist_item(&mut doc, &card_id, &checklist_id, &text)?;
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&item)?);
                } else {
                    println!("Added item: {} ({})", item.text, item.id);
                }
            }
            ChecklistCommands::ItemCheck { board_id, card_id, checklist_id, item_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::checklist::set_item_checked(&mut doc, &card_id, &checklist_id, &item_id, true)?;
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    println!("{}", serde_json::json!({"checked": true, "item_id": item_id}));
                } else {
                    println!("Checked item {item_id}");
                }
            }
            ChecklistCommands::ItemUncheck { board_id, card_id, checklist_id, item_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::checklist::set_item_checked(&mut doc, &card_id, &checklist_id, &item_id, false)?;
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    println!("{}", serde_json::json!({"checked": false, "item_id": item_id}));
                } else {
                    println!("Unchecked item {item_id}");
                }
            }
            ChecklistCommands::ItemDelete { board_id, card_id, checklist_id, item_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::checklist::delete_checklist_item(&mut doc, &card_id, &checklist_id, &item_id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    println!("{}", serde_json::json!({"deleted_item": item_id}));
                } else {
                    println!("Deleted checklist item {item_id}");
                }
            }
            ChecklistCommands::Delete { board_id, card_id, checklist_id, json } => {
                let mut doc = storage.load_board(&board_id)?;
                monotask_core::checklist::delete_checklist(&mut doc, &card_id, &checklist_id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    println!("{}", serde_json::json!({"deleted_checklist": checklist_id}));
                } else {
                    println!("Deleted checklist {checklist_id}");
                }
            }
        },
        Commands::Field { cmd } => match cmd {
            FieldCommands::Create { board_id, name, field_type, options, default_value, auto_apply, json } => {
                use monotask_core::field::FieldType;
                let ft = FieldType::from_str(&field_type)
                    .ok_or_else(|| anyhow::anyhow!("unknown field type '{}'. Valid types: text, number, date, select, multi_select, checkbox", field_type))?;
                let mut doc = storage.load_board(&board_id)?;
                let def = monotask_core::field::create_field(&mut doc, &name, ft, options, default_value, auto_apply)?;
                storage.save_board(&board_id, &mut doc)?;
                if json {
                    println!("{}", serde_json::json!({
                        "id": def.id, "name": def.name, "type": def.field_type.as_str(),
                        "options": def.options, "default_value": def.default_value,
                        "auto_apply": def.auto_apply,
                    }));
                } else {
                    println!("Created field: {} ({}) [{}]", def.name, def.id, def.field_type.as_str());
                }
            }
            FieldCommands::List { board_id, json } => {
                let doc = storage.load_board(&board_id)?;
                let fields = monotask_core::field::list_fields(&doc)?;
                let visible: Vec<_> = fields.iter().filter(|f| !f.archived).collect();
                if json {
                    let out: Vec<serde_json::Value> = visible.iter().map(|f| serde_json::json!({
                        "id": f.id, "name": f.name, "type": f.field_type.as_str(),
                        "options": f.options, "default_value": f.default_value,
                        "auto_apply": f.auto_apply,
                    })).collect();
                    println!("{}", serde_json::to_string_pretty(&out)?);
                } else if visible.is_empty() {
                    println!("No fields defined. Use `monotaskcli field create` to add one.");
                } else {
                    for f in &visible {
                        let dv = f.default_value.as_deref().map(|v| format!(" default={v}")).unwrap_or_default();
                        let aa = if f.auto_apply { " auto_apply" } else { "" };
                        println!("{} — {} [{}]{}{}", f.id, f.name, f.field_type.as_str(), dv, aa);
                    }
                }
            }
            FieldCommands::Rename { board_id, field_ref, new_name, json } => {
                let mut doc = storage.load_board(&board_id)?;
                let def = monotask_core::field::resolve_field_ref(&doc, &field_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .ok_or_else(|| anyhow::anyhow!("field '{}' not found", field_ref))?;
                monotask_core::field::rename_field(&mut doc, &def.id, &new_name)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"field_id": def.id, "name": new_name})); }
                else { println!("Renamed field {} to: {}", def.id, new_name); }
            }
            FieldCommands::Delete { board_id, field_ref, json } => {
                let mut doc = storage.load_board(&board_id)?;
                let def = monotask_core::field::resolve_field_ref(&doc, &field_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .ok_or_else(|| anyhow::anyhow!("field '{}' not found", field_ref))?;
                monotask_core::field::archive_field(&mut doc, &def.id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"archived": def.id})); }
                else { println!("Archived field {} ({})", def.name, def.id); }
            }
            FieldCommands::Backfill { board_id, field_ref, json } => {
                let mut doc = storage.load_board(&board_id)?;
                let def = monotask_core::field::resolve_field_ref(&doc, &field_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .ok_or_else(|| anyhow::anyhow!("field '{}' not found", field_ref))?;
                let count = monotask_core::field::backfill_field_defaults(&mut doc, &def.id)?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"field_id": def.id, "updated_count": count})); }
                else { println!("Backfilled {} cards with default value for '{}'", count, def.name); }
            }
            FieldCommands::Update { board_id, field_ref, default_value, auto_apply, json } => {
                let mut doc = storage.load_board(&board_id)?;
                let def = monotask_core::field::resolve_field_ref(&doc, &field_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .ok_or_else(|| anyhow::anyhow!("field '{}' not found", field_ref))?;
                monotask_core::field::update_field_default(
                    &mut doc, &def.id,
                    default_value.as_deref(),
                    auto_apply,
                )?;
                storage.save_board(&board_id, &mut doc)?;
                if json { println!("{}", serde_json::json!({"field_id": def.id, "ok": true})); }
                else { println!("Updated field {}", def.name); }
            }
        },
        Commands::Space { cmd } => handle_space(cmd, &mut storage, &identity)?,
        Commands::Profile { cmd } => handle_profile(cmd, &mut storage, &identity, &dir)?,
        Commands::Version => println!("monotaskcli {}", env!("CARGO_PKG_VERSION")),
        Commands::AiHelp => print_ai_help(),
        Commands::App { cmd } => match cmd {
            AppCommands::Open { url } => {
                if !url.starts_with("monotask://") {
                    anyhow::bail!("URL must start with monotask://");
                }
                open_url(&url);
                println!("Opening: {}", url);
            }
        },
        Commands::Chat { cmd } => {
            let chat_doc_id = |space_id: &str| format!("{space_id}-chat");
            match cmd {
                ChatCommands::Send { space_id, text, json } => {
                    let doc_id = chat_doc_id(&space_id);
                    let mut doc = match storage.load_board(&doc_id) {
                        Ok(d) => d,
                        Err(_) => monotask_core::chat::create_chat_doc()
                            .map_err(|e| anyhow::anyhow!(e))?,
                    };
                    let msg = monotask_core::chat::ChatMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        author: identity.public_key_hex(),
                        text: text.clone(),
                        created_at: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        refs: vec![],
                    };
                    monotask_core::chat::append_message(&mut doc, &msg)
                        .map_err(|e| anyhow::anyhow!(e))?;
                    let bytes = doc.save();
                    storage.save_board_bytes(&doc_id, &bytes, true)
                        .map_err(|e| anyhow::anyhow!(e))?;
                    if json { println!("{}", serde_json::to_string_pretty(&msg)?); }
                    else { println!("Sent: {}", msg.text); }
                }
                ChatCommands::List { space_id, limit, json } => {
                    let doc_id = chat_doc_id(&space_id);
                    let doc = match storage.load_board(&doc_id) {
                        Ok(d) => d,
                        Err(_) => {
                            if json { println!("[]"); } else { println!("No messages."); }
                            return Ok(());
                        }
                    };
                    let msgs = monotask_core::chat::list_messages(&doc, limit, None)
                        .map_err(|e| anyhow::anyhow!(e))?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&msgs)?);
                    } else if msgs.is_empty() {
                        println!("No messages.");
                    } else {
                        for m in &msgs {
                            println!("[{}] {}: {}", m.created_at, &m.author[..8], m.text);
                        }
                    }
                }
            }
        },
        Commands::Sync { detach, stop, status, port, peers } => {
            cmd_sync(dir, detach, stop, status, port, peers).await?;
        }
        Commands::Github { cmd } => {
            cmd_github(cmd, &dir, &mut storage, &identity).await?;
        }
        Commands::Linear { cmd } => {
            cmd_linear(cmd, &dir, &mut storage, &identity).await?;
        }
        Commands::Mail { cmd } => {
            cmd_mail(cmd, &dir, &mut storage, &identity).await?;
        }
    }
    Ok(())
}

async fn cmd_sync(
    data_dir: std::path::PathBuf,
    detach: bool,
    stop: bool,
    status: bool,
    port: u16,
    peers: Vec<String>,
) -> anyhow::Result<()> {
    use monotask_net::{NetworkHandle, NetConfig, NetEvent};
    use monotask_storage::Storage;
    use std::sync::{Arc, Mutex};

    let pid_file = data_dir.join("sync.pid");

    if stop {
        let pid_str = std::fs::read_to_string(&pid_file)
            .map_err(|_| anyhow::anyhow!("sync daemon not running (no PID file)"))?;
        let pid: u32 = pid_str.trim().parse()?;
        #[cfg(unix)]
        unsafe { libc::kill(pid as i32, libc::SIGTERM); }
        #[cfg(windows)]
        { std::process::Command::new("taskkill").args(["/F", "/PID", &pid.to_string()]).status().ok(); }
        println!("Stopped sync daemon (PID {pid})");
        return Ok(());
    }

    if status {
        match std::fs::read_to_string(&pid_file) {
            Ok(pid) => println!("Sync daemon running (PID {})", pid.trim()),
            Err(_) => println!("Sync daemon not running"),
        }
        return Ok(());
    }

    if detach {
        let exe = std::env::current_exe()?;
        let mut args: Vec<String> = std::env::args().collect();
        args.retain(|a| a != "--detach");
        let child = std::process::Command::new(exe)
            .args(&args[1..])
            .spawn()?;
        std::fs::write(&pid_file, child.id().to_string())?;
        println!("Sync daemon started (PID {})", child.id());
        return Ok(());
    }

    // Load identity bytes and space IDs
    let (identity_bytes, space_ids) = {
        let storage = Storage::open(&data_dir)?;
        let space_ids: Vec<String> = monotask_storage::space::list_spaces(storage.conn())?
            .into_iter().map(|s| s.id).collect();
        let key_path = data_dir.join("identity.key");
        let bytes = std::fs::read(&key_path)
            .map_err(|_| anyhow::anyhow!("Identity key not found. Run `monotaskcli init` first."))?;
        let identity_bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid key file length"))?;
        (identity_bytes, space_ids)
    };

    let storage = Arc::new(Mutex::new(Storage::open(&data_dir)?));
    let mut handle = NetworkHandle::start(
        NetConfig { listen_port: port, data_dir: data_dir.clone(), bootstrap_peers: peers },
        Arc::clone(&storage),
        identity_bytes,
    ).await?;

    handle.announce_spaces(space_ids).await;

    // Snapshot current last_modified timestamps so we can detect CLI-side changes.
    let mut last_seen: std::collections::HashMap<String, i64> = {
        let guard = storage.lock().unwrap();
        let mut stmt = guard.conn()
            .prepare("SELECT board_id, last_modified FROM boards")
            .unwrap();
        stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    // Use SyncTrigger so we can poll in the same select! that receives events.
    let sync_trigger = handle.sync_trigger();
    let mut poll_interval = tokio::time::interval(std::time::Duration::from_secs(2));

    println!("Sync daemon running. Press Ctrl+C to stop.");
    loop {
        tokio::select! {
            Some(event) = async { if let Some(rx) = handle.event_rx.as_mut() { rx.recv().await } else { None } } => {
                match event {
                    NetEvent::PeerConnected { peer_id } =>
                        println!("connected: {peer_id}"),
                    NetEvent::PeerDisconnected { peer_id } =>
                        println!("disconnected: {peer_id}"),
                    NetEvent::BoardSynced { board_id, peer_id } =>
                        println!("synced board {board_id} with {peer_id}"),
                    NetEvent::SyncError { board_id, error } =>
                        println!("sync error {board_id}: {error}"),
                }
            }
            _ = poll_interval.tick() => {
                // Detect boards modified by CLI commands (or any other process).
                let current: std::collections::HashMap<String, i64> = {
                    let guard = storage.lock().unwrap();
                    let mut stmt = guard.conn()
                        .prepare("SELECT board_id, last_modified FROM boards")
                        .unwrap_or_else(|_| panic!("failed to prepare board poll query"));
                    stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
                        .unwrap()
                        .filter_map(|r| r.ok())
                        .collect()
                };
                for (board_id, ts) in &current {
                    let old_ts = last_seen.get(board_id).copied().unwrap_or(0);
                    if *ts > old_ts {
                        println!("board {board_id:.8} changed (ts={ts}), triggering sync");
                        sync_trigger.trigger_sync(board_id.clone()).await;
                    }
                }
                last_seen = current;
            }
        }
    }
}

fn handle_space(cmd: SpaceCommands, storage: &mut monotask_storage::Storage, identity: &monotask_crypto::Identity) -> anyhow::Result<()> {
    use monotask_core::space as cs;
    use monotask_storage::space as ss;

    match cmd {
        SpaceCommands::Create { name } => {
            let space_id = uuid::Uuid::new_v4().to_string();
            let owner_pubkey = identity.public_key_hex();
            let mut doc = cs::create_space_doc(&name, &owner_pubkey)?;
            let profile = get_local_member_profile(storage.conn());
            cs::add_member(&mut doc, &owner_pubkey, &profile)?;
            let bytes = doc.save();
            ss::create_space(storage.conn(), &space_id, &name, &owner_pubkey, &bytes)?;
            let owner_member = cs::Member {
                pubkey: owner_pubkey.clone(),
                display_name: if profile.display_name.is_empty() { None } else { Some(profile.display_name.clone()) },
                avatar_blob: None,
                bio: None,
                role: None,
                color_accent: None,
                presence: None,
                kicked: false,
            };
            ss::upsert_member(storage.conn(), &space_id, &owner_member)?;
            println!("Created Space: {} ({})", name, space_id);
        }
        SpaceCommands::List => {
            let spaces = ss::list_spaces(storage.conn())?;
            if spaces.is_empty() {
                println!("No spaces found.");
            } else {
                for s in spaces {
                    println!("{} | {} | {} members", s.id, s.name, s.member_count);
                }
            }
        }
        SpaceCommands::Info { space_id } => {
            let space = ss::get_space(storage.conn(), &space_id)?;
            println!("Space: {} ({})", space.name, space.id);
            println!("Owner: {}", space.owner_pubkey);
            println!("Members ({}):", space.members.len());
            for m in &space.members {
                let name = m.display_name.as_deref().unwrap_or("(unnamed)");
                let kicked = if m.kicked { " [kicked]" } else { "" };
                println!("  {}  {}{}", &m.pubkey[..16], name, kicked);
            }
            println!("Boards ({}):", space.boards.len());
            for b in &space.boards {
                println!("  {}", b);
            }
        }
        SpaceCommands::Invite { cmd } => match cmd {
            SpaceInviteCommands::Generate { space_id } => {
                ss::revoke_all_invites(storage.conn(), &space_id)?;
                let doc_bytes = ss::load_space_doc(storage.conn(), &space_id)?;
                let token = monotask_crypto::generate_invite_token(&space_id, identity, Some(&doc_bytes))?;
                let meta = monotask_crypto::verify_invite_token_signature(&token)?;
                ss::insert_invite(storage.conn(), &meta.token_hash, &token, &space_id, None)?;
                println!("{}", token);
            }
            SpaceInviteCommands::Export { space_id, output_file } => {
                ss::revoke_all_invites(storage.conn(), &space_id)?;
                let doc_bytes = ss::load_space_doc(storage.conn(), &space_id)?;
                let token = monotask_crypto::generate_invite_token(&space_id, identity, Some(&doc_bytes))?;
                let meta = monotask_crypto::verify_invite_token_signature(&token)?;
                ss::insert_invite(storage.conn(), &meta.token_hash, &token, &space_id, None)?;
                let space = ss::get_space(storage.conn(), &space_id)?;
                use base64::Engine;
                let space_doc_b64 = base64::engine::general_purpose::STANDARD.encode(&doc_bytes);
                let payload = serde_json::json!({
                    "token": token,
                    "space_name": space.name,
                    "space_doc": space_doc_b64,
                });
                std::fs::write(&output_file, serde_json::to_string_pretty(&payload)?)?;
                println!("Exported invite to {}", output_file);
            }
            SpaceInviteCommands::Revoke { space_id } => {
                ss::revoke_all_invites(storage.conn(), &space_id)?;
                println!("Revoked all active invites for {}", space_id);
            }
        },
        SpaceCommands::Join { token_or_file } => {
            let local_pubkey = identity.public_key_hex();
            let (token, _hint_name, file_doc_opt) = parse_token_or_file(&token_or_file)?;
            let meta = monotask_crypto::verify_invite_token_signature(&token)?;
            ss::check_invite_policy(storage.conn(), &meta, &local_pubkey)?;

            // Prefer doc from token (v2), fall back to .space file payload, then stub
            let resolved_doc = meta.space_doc.clone().or(file_doc_opt);

            // If already a member but we now have a doc, update name + boards
            if let Ok(existing) = ss::get_space(storage.conn(), &meta.space_id) {
                if existing.members.iter().any(|m| m.pubkey == local_pubkey) {
                    if let Some(ref bytes) = resolved_doc {
                        let mut doc = automerge::AutoCommit::load(bytes)?;
                        let boards = cs::list_board_refs(&doc)?;
                        let members = cs::list_members(&doc)?;
                        let new_name = cs::get_space_name(&doc).unwrap_or(existing.name);
                        ss::update_space_doc(storage.conn(), &meta.space_id, &doc.save())?;
                        ss::rename_space(storage.conn(), &meta.space_id, &new_name)?;
                        for m in &members { let _ = ss::upsert_member(storage.conn(), &meta.space_id, m); }
                        for b in &boards { let _ = ss::add_board(storage.conn(), &meta.space_id, b); }
                        println!("Updated Space: {} ({})", new_name, meta.space_id);
                    } else {
                        println!("Already a member of Space: {} ({})", existing.name, meta.space_id);
                    }
                    return Ok(());
                }
            }
            let local_profile = get_local_member_profile(storage.conn());
            let (mut doc, members, boards, space_name) = if let Some(bytes) = resolved_doc {
                let doc = automerge::AutoCommit::load(&bytes)?;
                let name = cs::get_space_name(&doc).unwrap_or_else(|| "Shared Space".into());
                let members = cs::list_members(&doc)?;
                let boards = cs::list_board_refs(&doc)?;
                (doc, members, boards, name)
            } else {
                let mut doc = cs::create_space_doc("Shared Space", &meta.owner_pubkey)?;
                let empty = cs::MemberProfile { display_name: String::new(), avatar_b64: String::new(), bio: String::new(), role: String::new(), color_accent: String::new(), presence: String::new(), kicked: false };
                cs::add_member(&mut doc, &meta.owner_pubkey, &empty)?;
                let stub_owner = cs::Member {
                    pubkey: meta.owner_pubkey.clone(),
                    display_name: None,
                    avatar_blob: None,
                    bio: None,
                    role: None,
                    color_accent: None,
                    presence: None,
                    kicked: false,
                };
                (doc, vec![stub_owner], vec![], "Shared Space".into())
            };
            cs::add_member(&mut doc, &local_pubkey, &local_profile)?;
            let doc_bytes = doc.save();
            let _ = ss::create_space(storage.conn(), &meta.space_id, &space_name, &meta.owner_pubkey, &doc_bytes);
            for m in &members {
                let _ = ss::upsert_member(storage.conn(), &meta.space_id, m);
            }
            let local_sql = cs::Member {
                pubkey: local_pubkey,
                display_name: if local_profile.display_name.is_empty() { None } else { Some(local_profile.display_name) },
                avatar_blob: None,
                bio: None,
                role: None,
                color_accent: None,
                presence: None,
                kicked: false,
            };
            ss::upsert_member(storage.conn(), &meta.space_id, &local_sql)?;
            for b in &boards {
                let _ = ss::add_board(storage.conn(), &meta.space_id, b);
            }
            println!("Joined Space: {} ({})", space_name, meta.space_id);
        }
        SpaceCommands::Boards { cmd } => match cmd {
            SpaceBoardsCommands::Add { space_id, board_id } => {
                let bytes = ss::load_space_doc(storage.conn(), &space_id)?;
                let mut doc = automerge::AutoCommit::load(&bytes)?;
                cs::add_board_ref(&mut doc, &board_id)?;
                ss::update_space_doc(storage.conn(), &space_id, &doc.save())?;
                ss::add_board(storage.conn(), &space_id, &board_id)?;
                println!("Added board {} to Space {}", board_id, space_id);
            }
            SpaceBoardsCommands::Remove { space_id, board_id } => {
                let bytes = ss::load_space_doc(storage.conn(), &space_id)?;
                let mut doc = automerge::AutoCommit::load(&bytes)?;
                cs::remove_board_ref(&mut doc, &board_id)?;
                ss::update_space_doc(storage.conn(), &space_id, &doc.save())?;
                ss::remove_board(storage.conn(), &space_id, &board_id)?;
                println!("Removed board {} from Space {}", board_id, space_id);
            }
            SpaceBoardsCommands::List { space_id } => {
                let space = ss::get_space(storage.conn(), &space_id)?;
                for b in &space.boards { println!("{}", b); }
            }
        },
        SpaceCommands::Members { cmd } => match cmd {
            SpaceMembersCommands::List { space_id } => {
                let space = ss::get_space(storage.conn(), &space_id)?;
                for m in &space.members {
                    let name = m.display_name.as_deref().unwrap_or("(unnamed)");
                    let kicked = if m.kicked { " [kicked]" } else { "" };
                    println!("{}  {}{}", m.pubkey, name, kicked);
                }
            }
            SpaceMembersCommands::Kick { space_id, pubkey } => {
                let bytes = ss::load_space_doc(storage.conn(), &space_id)?;
                let mut doc = automerge::AutoCommit::load(&bytes)?;
                cs::kick_member(&mut doc, &pubkey)?;
                ss::update_space_doc(storage.conn(), &space_id, &doc.save())?;
                ss::set_member_kicked(storage.conn(), &space_id, &pubkey, true)?;
                println!("Kicked {} from Space {}", pubkey, space_id);
            }
        },
    }
    Ok(())
}

fn handle_profile(cmd: ProfileCommands, storage: &mut monotask_storage::Storage, identity: &monotask_crypto::Identity, data_dir: &std::path::Path) -> anyhow::Result<()> {
    use monotask_storage::space as ss;

    match cmd {
        ProfileCommands::Show => {
            let profile = ss::get_profile(storage.conn())?
                .unwrap_or_else(|| monotask_core::space::UserProfile {
                    pubkey: identity.public_key_hex(),
                    display_name: None,
                    avatar_blob: None,
                    bio: None,
                    role: None,
                    color_accent: None,
                    presence: None,
                    ssh_key_path: None,
                });
            println!("Pubkey:       {}", profile.pubkey);
            println!("Display name: {}", profile.display_name.as_deref().unwrap_or("(not set)"));
            println!("Avatar:       {}", if profile.avatar_blob.is_some() { "set" } else { "not set" });
            println!("SSH key path: {}", profile.ssh_key_path.as_deref().unwrap_or("(auto-generated)"));
        }
        ProfileCommands::SetName { name } => {
            let existing = ss::get_profile(storage.conn())?.unwrap_or_else(|| monotask_core::space::UserProfile {
                pubkey: identity.public_key_hex(),
                display_name: None,
                avatar_blob: None,
                bio: None,
                role: None,
                color_accent: None,
                presence: None,
                ssh_key_path: None,
            });
            ss::upsert_profile(storage.conn(), &monotask_core::space::UserProfile {
                display_name: Some(name.clone()),
                ..existing
            })?;
            println!("Display name set to: {}", name);
        }
        ProfileCommands::SetAvatar { path } => {
            let avatar_blob = std::fs::read(&path)?;
            let existing = ss::get_profile(storage.conn())?.unwrap_or_else(|| monotask_core::space::UserProfile {
                pubkey: identity.public_key_hex(),
                display_name: None,
                avatar_blob: None,
                bio: None,
                role: None,
                color_accent: None,
                presence: None,
                ssh_key_path: None,
            });
            ss::upsert_profile(storage.conn(), &monotask_core::space::UserProfile {
                avatar_blob: Some(avatar_blob),
                ..existing
            })?;
            println!("Avatar set from {}", path);
        }
        ProfileCommands::ImportSshKey { path } => {
            let path_ref = path.as_deref().map(std::path::Path::new);
            let new_identity = monotask_crypto::import_ssh_identity(path_ref)?;
            let pubkey = new_identity.public_key_hex();
            let key_bytes = new_identity.to_secret_bytes();
            std::fs::write(data_dir.join("identity.key"), key_bytes)?;
            let existing = ss::get_profile(storage.conn())?;
            ss::upsert_profile(storage.conn(), &monotask_core::space::UserProfile {
                pubkey: pubkey.clone(),
                display_name: existing.as_ref().and_then(|p| p.display_name.clone()),
                avatar_blob: existing.as_ref().and_then(|p| p.avatar_blob.clone()),
                bio: existing.as_ref().and_then(|p| p.bio.clone()),
                role: existing.as_ref().and_then(|p| p.role.clone()),
                color_accent: existing.as_ref().and_then(|p| p.color_accent.clone()),
                presence: existing.as_ref().and_then(|p| p.presence.clone()),
                ssh_key_path: path,
            })?;
            println!("Imported SSH key. New pubkey: {}", pubkey);
        }
    }
    Ok(())
}

fn get_local_member_profile(conn: &rusqlite::Connection) -> monotask_core::space::MemberProfile {
    use monotask_storage::space as ss;
    let profile = ss::get_profile(conn).ok().flatten();
    monotask_core::space::MemberProfile {
        display_name: profile.as_ref()
            .and_then(|p| p.display_name.clone())
            .unwrap_or_default(),
        avatar_b64: profile.as_ref()
            .and_then(|p| p.avatar_blob.as_ref())
            .map(|b| { use base64::Engine; base64::engine::general_purpose::STANDARD.encode(b) })
            .unwrap_or_default(),
        bio: "".into(),
        role: "".into(),
        color_accent: "".into(),
        presence: "".into(),
        kicked: false,
    }
}

async fn cmd_github(
    cmd: GithubCommands,
    data_dir: &std::path::Path,
    storage: &mut monotask_storage::Storage,
    identity: &monotask_crypto::Identity,
) -> anyhow::Result<()> {
    use colored::Colorize;
    match cmd {
        GithubCommands::Connect { token } => {
            let tok = match token {
                Some(t) => t,
                None => {
                    eprint!("Enter GitHub PAT: ");
                    tokio::task::spawn_blocking(|| {
                        let mut s = String::new();
                        std::io::stdin().read_line(&mut s).map(|_| s.trim().to_string())
                    }).await
                    .map_err(|e| anyhow::anyhow!("thread error: {e}"))?
                    .map_err(|e| anyhow::anyhow!("stdin error: {e}"))?
                }
            };
            if tok.is_empty() {
                anyhow::bail!("Token cannot be empty");
            }
            let valid = monotask_github::test_token(&tok).await.unwrap_or(false);
            if !valid {
                anyhow::bail!("Token validation failed — check the token and network access");
            }
            monotask_github::save_token(data_dir, &tok)?;
            println!("{}", "✓ Token saved and verified".green());
        }
        GithubCommands::Status => {
            match monotask_github::load_token(data_dir)? {
                Some(_) => println!("Token: {}", "saved".green()),
                None => println!("Token: {}", "not set — run `monotaskcli github connect`".yellow()),
            }
        }
        GithubCommands::Link { board_id, owner, repo, done_col } => {
            let mut doc = storage.load_board(&board_id)?;
            let config = monotask_github::GitHubConfig {
                owner: owner.clone(), repo: repo.clone(),
                done_column_id: done_col, last_sync: None,
            };
            monotask_github::set_github_config(&mut doc, Some(&config))?;
            storage.save_board(&board_id, &mut doc)?;
            println!("Linked board {board_id} → {owner}/{repo}");
        }
        GithubCommands::Unlink { board_id } => {
            let mut doc = storage.load_board(&board_id)?;
            monotask_github::set_github_config(&mut doc, None)?;
            storage.save_board(&board_id, &mut doc)?;
            println!("Unlinked board {board_id} from GitHub");
        }
        GithubCommands::Sync { board_id } => {
            let token = monotask_github::load_token(data_dir)?
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run `monotaskcli github connect` first."))?;
            let mut doc = storage.load_board(&board_id)?;
            let config = monotask_github::get_github_config(&doc)
                .ok_or_else(|| anyhow::anyhow!("Board not linked to GitHub. Run `monotaskcli github link` first."))?;
            let actor_pk = identity.public_key_bytes().to_vec();
            let result = monotask_github::sync_board(&mut doc, &token, &config, &actor_pk).await?;
            storage.save_board(&board_id, &mut doc)?;
            println!("Sync complete: ↑{} pushed  ↓{} pulled  ✕{} closed",
                result.pushed, result.pulled, result.closed);
            if !result.errors.is_empty() {
                eprintln!("{} non-fatal errors:", result.errors.len());
                for e in &result.errors { eprintln!("  - {e}"); }
            }
        }
    }
    Ok(())
}

async fn cmd_linear(
    cmd: LinearCommands,
    data_dir: &std::path::Path,
    storage: &mut monotask_storage::Storage,
    identity: &monotask_crypto::Identity,
) -> anyhow::Result<()> {
    use colored::Colorize;
    match cmd {
        LinearCommands::Connect { token } => {
            let tok = match token {
                Some(t) => t,
                None => {
                    eprint!("Enter Linear API key: ");
                    tokio::task::spawn_blocking(|| {
                        let mut s = String::new();
                        std::io::stdin().read_line(&mut s).map(|_| s.trim().to_string())
                    }).await
                    .map_err(|e| anyhow::anyhow!("thread error: {e}"))?
                    .map_err(|e| anyhow::anyhow!("stdin error: {e}"))?
                }
            };
            if tok.is_empty() {
                anyhow::bail!("Token cannot be empty");
            }
            let valid = monotask_linear::test_token(&tok).await.unwrap_or(false);
            if !valid {
                anyhow::bail!("Token validation failed — check the key and network access");
            }
            monotask_linear::save_token(data_dir, &tok)?;
            println!("{}", "✓ Linear API key saved and verified".green());
        }
        LinearCommands::Status => {
            match monotask_linear::load_token(data_dir)? {
                Some(tok) => {
                    println!("Token: {}", "saved".green());
                    match monotask_linear::list_teams(&tok).await {
                        Ok(teams) => {
                            println!("Teams ({}):", teams.len());
                            for t in &teams {
                                println!("  {} — {} (key: {})", t.id.dimmed(), t.name.bold(), t.key);
                            }
                        }
                        Err(e) => eprintln!("Could not fetch teams: {e}"),
                    }
                }
                None => println!("Token: {}", "not set — run `monotaskcli linear connect`".yellow()),
            }
        }
        LinearCommands::Teams => {
            let token = monotask_linear::load_token(data_dir)?
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run `monotaskcli linear connect` first."))?;
            let teams = monotask_linear::list_teams(&token).await?;
            println!("{:<40} {:<20} {}", "ID", "Key", "Name");
            for t in &teams {
                println!("{:<40} {:<20} {}", t.id, t.key, t.name);
            }
        }
        LinearCommands::Projects { team_id } => {
            let token = monotask_linear::load_token(data_dir)?
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run `monotaskcli linear connect` first."))?;
            let projects = monotask_linear::list_projects(&token, &team_id).await?;
            println!("{:<40} {}", "ID", "Name");
            for p in &projects {
                println!("{:<40} {}", p.id, p.name);
            }
        }
        LinearCommands::Link { board_id, team, project, done_col } => {
            let token = monotask_linear::load_token(data_dir)?
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run `monotaskcli linear connect` first."))?;
            let mut doc = storage.load_board(&board_id)?;

            // Fetch project name for display
            let projects = monotask_linear::list_projects(&token, &team).await?;
            let project_name = projects.iter()
                .find(|p| p.id == project)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| project.clone());

            println!("Setting up columns from Linear workflow states…");
            let (done_col_id, done_state_id) = monotask_linear::setup_columns_from_states(
                &mut doc,
                &token,
                &team,
                done_col.as_deref(),
            ).await?;

            let config = monotask_linear::LinearConfig {
                team_id: team.clone(),
                project_id: project.clone(),
                project_name: project_name.clone(),
                done_column_id: done_col_id.clone(),
                done_state_id,
                last_sync: None,
            };
            monotask_linear::set_linear_config(&mut doc, Some(&config))?;
            storage.save_board(&board_id, &mut doc)?;

            let cols = monotask_core::column::list_columns(&doc)?;
            let done_title = cols.iter().find(|c| c.id == done_col_id)
                .map(|c| c.title.as_str()).unwrap_or("?");
            println!("{}", format!("Linked board {board_id} → {project_name}").green());
            println!("  Done column: {} ({})", done_title, done_col_id.dimmed());
        }
        LinearCommands::Unlink { board_id } => {
            let mut doc = storage.load_board(&board_id)?;
            monotask_linear::set_linear_config(&mut doc, None)?;
            storage.save_board(&board_id, &mut doc)?;
            println!("Unlinked board {board_id} from Linear");
        }
        LinearCommands::Sync { board_id } => {
            let token = monotask_linear::load_token(data_dir)?
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run `monotaskcli linear connect` first."))?;
            let mut doc = storage.load_board(&board_id)?;
            let config = monotask_linear::get_linear_config(&doc)
                .ok_or_else(|| anyhow::anyhow!("Board not linked to Linear. Run `monotaskcli linear link` first."))?;
            let actor_pk = identity.public_key_bytes().to_vec();
            let result = monotask_linear::sync_board(&mut doc, &token, &config, &actor_pk).await?;
            storage.save_board(&board_id, &mut doc)?;
            println!("Sync complete: ↑{} pushed  ↓{} pulled  ✕{} closed",
                result.pushed, result.pulled, result.closed);
            if !result.errors.is_empty() {
                eprintln!("{} non-fatal errors:", result.errors.len());
                for e in &result.errors { eprintln!("  - {e}"); }
            }
        }
    }
    Ok(())
}

async fn cmd_mail(
    cmd: MailCommands,
    data_dir: &std::path::Path,
    storage: &mut monotask_storage::Storage,
    identity: &monotask_crypto::Identity,
) -> anyhow::Result<()> {
    use colored::Colorize;
    match cmd {
        MailCommands::GmailConnect { client_id } => {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
            let port = listener.local_addr()?.port();
            let redirect_uri = format!("http://127.0.0.1:{port}/callback");
            let (verifier, challenge) = monotask_mail::generate_pkce();
            let url = monotask_mail::build_auth_url("gmail", &client_id, "common", &challenge, &redirect_uri)?;
            println!("Opening browser for Gmail authorization…");
            println!("If the browser doesn't open, visit:\n  {url}");
            open_url(&url);
            monotask_mail::wait_and_complete_oauth(listener, data_dir, "gmail", &client_id, "common", &verifier, &redirect_uri).await?;
            println!("{}", "✓ Gmail connected".green());
        }
        MailCommands::OutlookConnect { client_id, tenant_id } => {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
            let port = listener.local_addr()?.port();
            let redirect_uri = format!("http://127.0.0.1:{port}/callback");
            let (verifier, challenge) = monotask_mail::generate_pkce();
            let url = monotask_mail::build_auth_url("outlook", &client_id, &tenant_id, &challenge, &redirect_uri)?;
            println!("Opening browser for Outlook authorization…");
            println!("If the browser doesn't open, visit:\n  {url}");
            open_url(&url);
            monotask_mail::wait_and_complete_oauth(listener, data_dir, "outlook", &client_id, &tenant_id, &verifier, &redirect_uri).await?;
            println!("{}", "✓ Outlook connected".green());
        }
        MailCommands::Status => {
            let gmail = if monotask_mail::token_saved(data_dir, "gmail") { "connected".green() } else { "not connected".yellow() };
            let outlook = if monotask_mail::token_saved(data_dir, "outlook") { "connected".green() } else { "not connected".yellow() };
            println!("Gmail:   {gmail}");
            println!("Outlook: {outlook}");
        }
        MailCommands::Disconnect { provider } => {
            monotask_mail::delete_token(data_dir, &provider)?;
            println!("Disconnected {provider}");
        }
        MailCommands::ImapConnect { host, port, username, password, folder } => {
            let pwd = match password {
                Some(p) => p,
                None => {
                    eprint!("Password (or app password): ");
                    tokio::task::spawn_blocking(|| {
                        let mut s = String::new();
                        std::io::stdin().read_line(&mut s).map(|_| s.trim().to_string())
                    }).await
                    .map_err(|e| anyhow::anyhow!("thread error: {e}"))?
                    .map_err(|e| anyhow::anyhow!("stdin error: {e}"))?
                }
            };
            let creds = monotask_mail::ImapCredentials { host, port, username: username.clone(), password: pwd, folder };
            // Test connection before saving
            println!("Testing IMAP connection to {}:{}…", creds.host, creds.port);
            let test_creds = creds.clone();
            let test_result = tokio::task::spawn_blocking(move || {
                monotask_mail::imap_client::fetch_since_sync_test(&test_creds)
            }).await.map_err(|e| anyhow::anyhow!("thread error: {e}"))?;
            match test_result {
                Ok(()) => {
                    monotask_mail::save_imap_credentials(data_dir, &creds)?;
                    println!("{}", "✓ IMAP connected and credentials saved".green());
                    println!("  Username: {username}");
                    println!("  Tip: use `monotaskcli mail link <BOARD_ID> --provider imap` to link a board.");
                }
                Err(e) => {
                    anyhow::bail!("IMAP connection test failed: {e}\nCheck host, port, username, and password.");
                }
            }
        }
        MailCommands::ImapStatus => {
            if monotask_mail::imap_credentials_saved(data_dir) {
                if let Ok(Some(c)) = monotask_mail::load_imap_credentials(data_dir) {
                    println!("IMAP: {} ({}:{})", "connected".green(), c.host, c.port);
                    println!("  Username: {}", c.username);
                    println!("  Folder:   {}", c.folder);
                }
            } else {
                println!("IMAP: {}", "not configured — run `monotaskcli mail imap-connect`".yellow());
            }
        }
        MailCommands::ImapDisconnect => {
            monotask_mail::delete_imap_credentials(data_dir)?;
            println!("IMAP credentials removed.");
        }
        MailCommands::Link { board_id, provider, gmail_client_id, outlook_client_id, tenant_id, inbox_col, keep_last } => {
            let mut doc = storage.load_board(&board_id)?;
            // Arg > env var > existing board config (preserve on re-link)
            let existing = monotask_mail::get_mail_config(&doc);
            let resolve_id = |arg: Option<String>, env_key: &str, existing_val: Option<String>| {
                arg
                    .filter(|s| !s.is_empty())
                    .or_else(|| std::env::var(env_key).ok().filter(|s| !s.is_empty()))
                    .or(existing_val)
            };
            let config = monotask_mail::MailConfig {
                provider: provider.clone(),
                gmail_client_id: resolve_id(gmail_client_id, "MAIL_GMAIL_CLIENT_ID",
                    existing.as_ref().and_then(|c| c.gmail_client_id.clone())),
                outlook_client_id: resolve_id(outlook_client_id, "MAIL_OUTLOOK_CLIENT_ID",
                    existing.as_ref().and_then(|c| c.outlook_client_id.clone())),
                outlook_tenant_id: tenant_id
                    .filter(|s| !s.is_empty())
                    .or_else(|| std::env::var("MAIL_OUTLOOK_TENANT_ID").ok())
                    .or_else(|| existing.as_ref().map(|c| c.outlook_tenant_id.clone()))
                    .unwrap_or_else(|| "common".into()),
                inbox_col_id: inbox_col,
                keep_last,
                last_sync: existing.and_then(|c| c.last_sync),
            };
            let needs_client_id = (provider.contains("gmail") || provider == "both") && config.gmail_client_id.is_none()
                || (provider.contains("outlook") || provider == "both") && config.outlook_client_id.is_none();
            if needs_client_id {
                eprintln!("Warning: client ID not set for provider '{provider}'. Sync will skip that provider.");
                eprintln!("Re-run with --gmail-client-id or --outlook-client-id, or set MAIL_GMAIL_CLIENT_ID / MAIL_OUTLOOK_CLIENT_ID env vars.");
            }
            monotask_mail::set_mail_config(&mut doc, Some(&config))?;
            storage.save_board(&board_id, &mut doc)?;
            println!("Linked board {board_id} to email sync (provider: {provider})");
        }
        MailCommands::Unlink { board_id } => {
            let mut doc = storage.load_board(&board_id)?;
            monotask_mail::set_mail_config(&mut doc, None)?;
            storage.save_board(&board_id, &mut doc)?;
            println!("Unlinked board {board_id} from email sync");
        }
        MailCommands::Sync { board_id } => {
            let mut doc = storage.load_board(&board_id)?;
            let config = monotask_mail::get_mail_config(&doc)
                .ok_or_else(|| anyhow::anyhow!("Board not linked to email. Run `monotaskcli mail link` first."))?;
            let actor_pk = identity.public_key_bytes().to_vec();
            let result = monotask_mail::sync_board(&mut doc, data_dir, &config, &actor_pk).await?;
            storage.save_board(&board_id, &mut doc)?;
            println!("Sync complete: {} new contacts, {} updated, {} emails added",
                result.contacts_created, result.contacts_updated, result.emails_added);
        }
    }
    Ok(())
}

fn open_url(url: &str) {
    #[cfg(target_os = "macos")]
    { let _ = std::process::Command::new("open").arg(url).spawn(); }
    #[cfg(target_os = "linux")]
    { let _ = std::process::Command::new("xdg-open").arg(url).spawn(); }
    #[cfg(target_os = "windows")]
    { let _ = std::process::Command::new("cmd").args(["/c", "start", "", url]).spawn(); }
}

fn print_ai_help() {
    print!("{}", r##"
================================================================================
MONOTASK CLI – AI AGENT REFERENCE
================================================================================
Binary : monotask
Version: 1.3.0
Purpose: P2P task manager with local-first CRDT storage. Designed for
         task management, collaborative workspaces, and automation via CLI.

Run `monotaskcli ai-help` to print this document at any time.

--------------------------------------------------------------------------------
QUICK-START FOR AGENTS
--------------------------------------------------------------------------------
1. Check your identity:       monotaskcli profile show
2. Create or pick a space:    monotaskcli space create "My Team"
3. Create a board in a space: monotaskcli board create "My Project" --space <SPACE_ID>
4. List columns on a board:   monotaskcli column list <BOARD_ID>
5. Create a card:             monotaskcli card create <BOARD_ID> <COL_ID> "Task title"
6. View a card:               monotaskcli card view <BOARD_ID> <CARD_ID>
7. Add a comment:             monotaskcli card comment add <BOARD_ID> <CARD_ID> "text"

NOTE: Boards must belong to a space. `board create` requires --space <SPACE_ID>.
      Run `monotaskcli space list` first to get a space ID.

Always use --json for machine-readable output when parsing results.

--------------------------------------------------------------------------------
GLOBAL FLAGS
--------------------------------------------------------------------------------
--data-dir <PATH>
    Override the storage directory (default: $XDG_DATA_HOME/monotaskcli or
    ~/.local/share/monotaskcli on Linux/macOS).
    The directory contains:
      monotaskcli.db  – SQLite database (boards, spaces, profile, invites)
      identity.key – Raw 32-byte Ed25519 secret key (auto-created on first run)

--------------------------------------------------------------------------------
IDENTITY & AUTHENTICATION
--------------------------------------------------------------------------------
Every user has an Ed25519 keypair. The public key (hex, 64 chars) is your
persistent identity across all operations.

Identity resolution order (first found wins):
  1. SSH Ed25519 key at path stored in profile (set via `profile import-ssh-key`)
  2. identity.key file in data directory
  3. Auto-generated key (written to identity.key on first run)

Your public key is used as:
  - Space ownership and membership
  - Card authorship (created_by field)
  - Invite token signing/verification

--------------------------------------------------------------------------------
COMMANDS
--------------------------------------------------------------------------------

## init
Usage: monotaskcli init
Effect: Prints the data directory path. Triggers identity creation if missing.
        Safe to run multiple times (idempotent).

## version
Usage: monotaskcli version
Effect: Prints the CLI version string.

────────────────────────────────────────────────────────────────────────────────
## board
Boards are the top-level containers. Each board holds an ordered list of
columns; each column holds an ordered list of cards. Boards are stored as
Automerge CRDT documents (binary blobs in SQLite).

### board create <TITLE> --space <SPACE_ID>
  --space  SPACE_ID   (REQUIRED) The space this board belongs to.
  --json              Output JSON

  Creates a new board inside the given space. Boards must belong to a space —
  this is enforced at creation time. The board is added to the space immediately.

  Run `monotaskcli space list` to find your SPACE_ID before calling this.

  Text output:  "Created board: <title> (<id>) in space <space_id>"
  JSON output:  {"id":"<uuid>","title":"<title>","space_id":"<uuid>","deep_link":"monotask://board/<id>"}

  Error: if SPACE_ID does not exist locally, the command fails with:
    "Space '<id>' not found. Run `monotaskcli space list` to see available spaces."

  Example:
    $ SPACE=$(monotaskcli space list | awk 'NR==1{print $1}')
    $ monotaskcli board create "Sprint 42" --space $SPACE --json
    {"id":"a1b2c3d4-...","title":"Sprint 42","space_id":"...","deep_link":"monotask://board/a1b2c3..."}

### board list
  --json   Output JSON

  Lists all boards stored locally with their titles.
  Text output:  "<id>: <title>"  (one per line)
  JSON output:  [{"id":"<uuid>","title":"<str>"}, ...]

### board rename <BOARD_ID> <NEW_TITLE>
  --json   Output JSON

  Renames an existing board.
  Text output:  "Renamed board <id> to: <new_title>"
  JSON output:  {"board_id":"<uuid>","title":"<new_title>"}

### board delete <BOARD_ID> --space <SPACE_ID>
  --space  Space ID the board belongs to (required)
  --json   Output JSON

  Permanently deletes a board and removes it from its space.
  Removes board data, space membership, and the Automerge space doc ref.
  Use `monotaskcli board list` and `monotaskcli space list` to get IDs.
  Text output:  "Deleted board <id> from space <space_id>"
  JSON output:  {"deleted":true,"board_id":"<uuid>","space_id":"<uuid>"}

  Example:
    $ SPACE=$(monotaskcli space list --json | jq -r '.[0].id')
    $ BOARD=$(monotaskcli board list --json | jq -r '.[0].id')
    $ monotaskcli board delete $BOARD --space $SPACE

────────────────────────────────────────────────────────────────────────────────
## column
Columns are ordered within a board. Each column has an ID and a title and
maintains an ordered list of card IDs.

### column create <BOARD_ID> <TITLE>
  --json   Output JSON

  Creates a new column in the specified board.
  Text output:  "Created column: <title> (<id>)"
  JSON output:  {"id":"<uuid>","board_id":"<board_id>"}

### column list <BOARD_ID>
  --json   Output JSON

  Lists all columns in the board in order.
  Text output:  "<col_id>: <title>"  (one per line)
  JSON output:  [{"id":"...","title":"...","card_ids":["..."]}, ...]

  Note: card_ids is the ordered list of card UUIDs in each column.

### column rename <BOARD_ID> <COL_ID> <NEW_TITLE>
  --json   Output JSON

  Renames a column.
  Text output:  "Renamed column <id> to: <new_title>"
  JSON output:  {"col_id":"<uuid>","title":"<new_title>"}

### column delete <BOARD_ID> <COL_ID>
  --json   Output JSON

  Deletes a column and all card references in it (card data is soft-deleted).
  Text output:  "Deleted column <id>"
  JSON output:  {"deleted":"<uuid>"}

────────────────────────────────────────────────────────────────────────────────
## card
Cards are the primary work items. Each card belongs to exactly one column.

Card fields:
  id             – UUID (use this for all card operations)
  number         – Human-readable short ID "<prefix>-<seq>" e.g. "a7f3-1"
                   Prefix = first 4 chars of base32-encoded creator pubkey.
                   Sequence = per-creator counter (1, 2, 3, ...).
  title          – Short summary string
  description    – Long-form markdown text (may be empty)
  cover_color    – Optional CSS color string for the card header
  assignees      – List of pubkey strings
  labels         – List of label strings
  due_date       – Optional date string "YYYY-MM-DD" or null
  archived       – Boolean (soft-archive, hidden from normal views)
  deleted        – Boolean (soft-delete, hidden from all views)
  copied_from    – UUID of source card if this card was copied, else null
  created_by     – Hex pubkey of creator
  created_at     – HLC timestamp (see TIMESTAMPS section)
  impact         – Optional score 0–10 for ICE/weighted priority
  effort         – Optional score 0–10 for ICE/weighted priority
  direct_priority– Optional score 0–10 set without impact/effort
  github_issue_number – Linked GitHub issue number (set by github sync)
  parent         – {board_id, card_id} of parent card or null
  subtasks       – [{board_id, card_id}, ...] of child cards

Priority calculation (when impact and effort are set):
  priority = floor((impact + 10 - effort) / 2)   range: 0–10

### card list <BOARD_ID>
  --col <COL_ID>     Only return cards in this column
  --label <LABEL>    Only return cards that have this label (exact string match)
  --json             Output JSON

  Lists all non-deleted, non-archived cards. Both filters are optional and
  can be combined. Filters are applied server-side before any output.
  Text output:  "[<col_title>] <number> – <title> (<id>)"  per card
  JSON output:  array of card objects, each extended with "col_id" and "col_title"

  JSON schema per item (abbreviated):
    {
      "id": "<uuid>",
      "title": "<str>",
      "col_id": "<uuid>",
      "col_title": "<str>",
      "number": {"prefix":"<str>","seq":<int>} | null,
      "due_date": "<YYYY-MM-DD>" | null,
      "labels": ["<str>", ...],
      "impact": <int> | null,
      "effort": <int> | null,
      "direct_priority": <int> | null
    }

  Examples:
    $ monotaskcli card list $BOARD --json
    $ monotaskcli card list $BOARD --col $TODO_COL --json
    $ monotaskcli card list $BOARD --label "role:writer" --json
    $ monotaskcli card list $BOARD --col $COL --label "role:reviewer" --json

### card create <BOARD_ID> <COL_ID> <TITLE>
  --json   Output JSON

  Creates a card in the specified column.
  Text output:  "Created card: <title> (<id>)"
  JSON output:  {"id":"<uuid>","title":"<title>","board_id":"<board_id>","number":"<prefix>-<n>"}

### card view <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Reads and prints all fields of a single card, including parent and subtasks.
  Text output:  labelled key-value lines
  JSON output:  full Card struct plus "parent" and "subtasks" fields

  JSON schema (abbreviated):
    {
      "id": "<uuid>",
      "number": {"prefix":"<str>","seq":<int>} | null,
      "title": "<str>",
      "description": "<str>",
      "cover_color": "<str>" | null,
      "assignees": ["<pubkey>", ...],
      "labels": ["<str>", ...],
      "due_date": "<YYYY-MM-DD>" | null,
      "archived": false,
      "deleted": false,
      "copied_from": "<uuid>" | null,
      "created_by": "<hex-pubkey>",
      "created_at": "<hlc-timestamp>",
      "impact": <int> | null,
      "effort": <int> | null,
      "direct_priority": <int> | null,
      "github_issue_number": <int> | null,
      "parent": {"board_id":"<uuid>","card_id":"<uuid>"} | null,
      "subtasks": [{"board_id":"<uuid>","card_id":"<uuid>"}, ...]
    }

### card rename <BOARD_ID> <CARD_ID> <NEW_TITLE>
  --json   Output JSON

  Renames a card.
  Text output:  "Renamed card <id> to: <new_title>"
  JSON output:  {"card_id":"<uuid>","title":"<new_title>"}

### card delete <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Soft-deletes a card (deleted=true; hidden from all views).
  Text output:  "Deleted card <id>"
  JSON output:  {"deleted":"<uuid>"}

### card archive <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Soft-archives a card (archived=true; hidden from normal views).
  Text output:  "Archived card <id>"
  JSON output:  {"archived":"<uuid>"}

### card copy <BOARD_ID> <CARD_ID> <TARGET_COL_ID>
  --json   Output JSON

  Copies a card into the given column (same board). The new card has
  copied_from set to the source card's ID.
  Text output:  "Copied card to: <title> (<new_id>)"
  JSON output:  {"id":"<uuid>","title":"<str>"}

### card move <BOARD_ID> <CARD_ID> <TO_COL_ID>
  --json   Output JSON

  Moves a card to a different column (same board). Auto-detects current column.
  Text output:  "Moved card <id> to column <to_col_id>"
  JSON output:  {"card_id":"<uuid>","to_col_id":"<uuid>"}

### card set-description <BOARD_ID> <CARD_ID> <TEXT>
  --json   Output JSON

  Sets the card's long-form description (markdown supported).
  Text output:  "Updated description for card <id>"
  JSON output:  {"card_id":"<uuid>","description":"<str>"}

### card set-cover <BOARD_ID> <CARD_ID> <COLOR>
  --json   Output JSON

  Sets the card cover color. Use "none" to clear it.
  COLOR: any CSS color string, e.g. "#e74c3c" or "red".
  Text output:  "Set cover color for card <id>"
  JSON output:  {"card_id":"<uuid>","color":"<str>"}

### card set-due-date <BOARD_ID> <CARD_ID> <DATE>
  --json   Output JSON

  Sets the due date. DATE format: "YYYY-MM-DD". Use "none" to clear.
  Text output:  "Set due date for card <id>"
  JSON output:  {"card_id":"<uuid>","due_date":"<YYYY-MM-DD>" | null}

### card set-priority <BOARD_ID> <CARD_ID> <PRIORITY>
  --json   Output JSON

  Sets a legacy string priority label. Use "none" to clear.
  Prefer set-impact/set-effort or set-direct-priority for numeric scoring.
  Text output:  "Set priority for card <id>"
  JSON output:  {"card_id":"<uuid>","priority":"<str>"}

### card set-impact <BOARD_ID> <CARD_ID> <VALUE>
  --json   Output JSON

  Sets the impact score (0–10). Priority is recomputed as
  floor((impact + 10 - effort) / 2) and displayed immediately.
  Text output:  "Impact=<n>, Effort=<n> → Priority=<n>"
  JSON output:  {"card_id":"<uuid>","impact":<int>,"effort":<int>,"priority":<int>}

### card set-effort <BOARD_ID> <CARD_ID> <VALUE>
  --json   Output JSON

  Sets the effort score (0–10). Priority is recomputed immediately.
  Text output:  "Impact=<n>, Effort=<n> → Priority=<n>"
  JSON output:  {"card_id":"<uuid>","impact":<int>,"effort":<int>,"priority":<int>}

### card set-direct-priority <BOARD_ID> <CARD_ID> [VALUE]
  --clear  Remove direct priority instead of setting it
  --json   Output JSON

  Sets or clears a direct priority (0–10) bypassing impact/effort calculation.
  Conflicts with --clear: provide either VALUE or --clear, not both.
  Text output:  "Priority=<n>/10"  or  "Priority cleared"
  JSON output:  {"card_id":"<uuid>","direct_priority":<int> | null}

### card clear-priority <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Clears all scoring fields: impact, effort, and direct_priority simultaneously.
  Use this to fully reset a card to unscored state.
  Text output:  "Impact, effort and priority cleared for card <id>"
  JSON output:  {"card_id":"<uuid>","cleared":true}

### card set-assignee <BOARD_ID> <CARD_ID> <PUBKEY>
  --json   Output JSON

  Assigns a card to the user with the given hex pubkey. Use "none" to clear.
  Text output:  "Set assignee for card <id>"
  JSON output:  {"card_id":"<uuid>","assignee":"<pubkey>"}

### card attach-image <BOARD_ID> <CARD_ID> <FILE>
  --json   Output JSON

  Reads an image file and stores it as a base64-encoded attachment on the card.
  The attachment gets a short ID (6-char hex) usable as "img:<id>" in markdown.
  Supported formats: png, jpg/jpeg, gif, webp, svg.
  Text output:  "Attached <name> as img:<id> — embed with ![<name>](img:<id>)"
  JSON output:  {"id":"<6char>","name":"<filename>","mime":"<mimetype>","token":"img:<id>"}

### card list-attachments <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Lists all attachments on a card with ID, filename, mime type, and size.
  Text output:  "  img:<id>  <name>  (<mime>, ~<n>KB)"  per attachment
  JSON output:  [{"id":"<str>","name":"<str>","mime":"<str>","size_b64":<int>}, ...]

### card save-attachment <BOARD_ID> <CARD_ID> <ATTACHMENT_ID>
  --output <PATH>   Save to a specific path (default: original filename)
  --json            Output JSON

  Decodes and saves an attachment to disk.
  Text output:  "Saved <name> (<n> bytes) to <path>"
  JSON output:  {"saved":"<path>","size":<int>}

────────────────────────────────────────────────────────────────────────────────
## card label
Label management for cards (free-form strings).

### card label add <BOARD_ID> <CARD_ID> <LABEL>
  --json   Output JSON

  Adds a label string to the card.
  Text output:  "Added label '<label>' to card <id>"
  JSON output:  {"card_id":"<uuid>","label":"<str>"}

### card label remove <BOARD_ID> <CARD_ID> <LABEL>
  --json   Output JSON

  Removes a label string from the card.
  JSON output:  {"card_id":"<uuid>","removed_label":"<str>"}

### card label list <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Lists all labels on a card.
  Text output:  one label per line
  JSON output:  ["<str>", ...]

────────────────────────────────────────────────────────────────────────────────
## card comment
Comment thread management for cards.

### card comment add <BOARD_ID> <CARD_ID> <TEXT>
  --json   Output JSON

  Adds a comment to the card. Author field is the local identity public key (hex).
  JSON output:  {"id":"<uuid>","author":"<pubkey-hex>","text":"<str>",
                 "created_at":"<hlc>","deleted":false,
                 "image_b64":null,"image_mime":null,"image_name":null}

### card comment list <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Lists all non-deleted comments in chronological order.
  Text output:  "[<created_at>] <author>: <text>"
                Appends " [+image]" when the comment has an embedded image.
  JSON output:  array of comment objects (image_b64 included when present)

### card comment delete <BOARD_ID> <CARD_ID> <COMMENT_ID>
  --json   Output JSON

  Soft-deletes a comment (deleted=true, not returned in list).
  JSON output:  {"deleted":"<comment_id>"}

### card comment edit <BOARD_ID> <CARD_ID> <COMMENT_ID> <NEW_TEXT>
  --json   Output JSON

  Replaces the text of an existing comment.
  JSON output:  {"edited":"<comment_id>"}

────────────────────────────────────────────────────────────────────────────────
## card subtask
Subtask management — links cards in a parent/child hierarchy.

### card subtask add <PARENT_BOARD_ID> <PARENT_CARD_ID> <CHILD_BOARD_ID> <COL_ID> <TITLE>
  --json   Output JSON

  Creates a new card in CHILD_BOARD_ID / COL_ID and links it as a subtask of
  the parent card. The new card's parent field is set automatically.
  CHILD_BOARD_ID may equal PARENT_BOARD_ID (single-board subtasks).
  Text output:  "Created subtask: <title> (<id>) in board <board_id>"
  JSON output:  {"id":"<uuid>","title":"<str>","board_id":"<uuid>"}

### card subtask list <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Lists all subtask references of a card.
  Text output:  "<card_id> (board: <board_id>)"  per subtask
  JSON output:  [{"board_id":"<uuid>","card_id":"<uuid>"}, ...]

────────────────────────────────────────────────────────────────────────────────
## card prerequisite
Prerequisite management — declares that one card must be done before another.

### card prerequisite add <BOARD_ID> <CARD_ID> <PREREQ_BOARD_ID> <PREREQ_CARD_ID>
  --json   Output JSON

  Marks PREREQ_CARD_ID as a prerequisite of CARD_ID.
  A card cannot be its own prerequisite.
  Text output:  "Added prerequisite <prereq_card_id> (board: <prereq_board_id>) to card <card_id>"
  JSON output:  {"board_id":"<uuid>","card_id":"<uuid>",
                 "prereq_board_id":"<uuid>","prereq_card_id":"<uuid>"}

### card prerequisite list <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Lists all prerequisite references for a card.
  Text output:  "<card_id> (board: <board_id>)"  per prerequisite
  JSON output:  [{"board_id":"<uuid>","card_id":"<uuid>"}, ...]

### card prerequisite remove <BOARD_ID> <CARD_ID> <PREREQ_BOARD_ID> <PREREQ_CARD_ID>
  --json   Output JSON

  Removes a prerequisite link.
  JSON output:  {"ok":true}

### card link add <BOARD_ID> <CARD_ID> <TARGET_BOARD_ID> <TARGET_CARD_ID>
  --json   Output JSON

  Creates a directional card-to-card link stored in the CRDT document.
  JSON output:  {"ok":true,"board_id":"<str>","card_id":"<str>","target_board_id":"<str>","target_card_id":"<str>"}

### card link list <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Lists all outgoing card links for the card.
  JSON output:  [{"board_id":"<str>","card_id":"<str>"}, ...]

### card link remove <BOARD_ID> <CARD_ID> <TARGET_BOARD_ID> <TARGET_CARD_ID>
  --json   Output JSON

  Removes a card-to-card link.
  JSON output:  {"ok":true}

### card mentions <BOARD_ID> <CARD_ID>
  --json   Output JSON

  Returns all @mention tokens found in the card's description (indexed in SQLite).
  Text output:  @alice (2026-06-30T12:00:00Z)
  JSON output:  [{"mention":"alice","created_at":"<ISO8601>"}, ...]

────────────────────────────────────────────────────────────────────────────────
## field
Custom field definitions are board-scoped typed key-value schemas stored in the
Automerge document. Field values are stored on each card and indexed in SQLite.

### field create <BOARD_ID> <NAME>
  --field-type <TYPE>      text (default), number, date, select, multi_select, checkbox
  --option <VALUE>         Allowed option (repeat; required for select/multi_select)
  --default-value <VAL>    Default written when --auto-apply is set
  --auto-apply             Apply default to every new card automatically
  --json                   Output JSON

  Creates a new field definition. Fields accept names or UUIDs in all other commands.
  JSON output:  {"id":"<uuid>","name":"<str>","type":"<str>","options":[...],"default_value":<str|null>,"auto_apply":<bool>}

### field list <BOARD_ID>
  --json

  Lists all non-archived field definitions on a board.
  JSON output:  [{"id":"<uuid>","name":"<str>","type":"<str>","options":[...],"default_value":<str|null>,"auto_apply":<bool>}, ...]

### field rename <BOARD_ID> <FIELD_REF> <NEW_NAME>
  --json

  Renames a field. FIELD_REF may be field name (case-insensitive) or UUID.
  JSON output:  {"field_id":"<uuid>","name":"<str>"}

### field update <BOARD_ID> <FIELD_REF>
  --default-value <VAL>    New default value
  --auto-apply <true|false>
  --json

  Updates a field's default value and/or auto-apply flag without renaming.
  JSON output:  {"field_id":"<uuid>","ok":true}

### field delete <BOARD_ID> <FIELD_REF>
  --json

  Archives (soft-deletes) a field. Archived fields are hidden from list/schema
  but existing card values are preserved.
  JSON output:  {"archived":"<uuid>"}

### field backfill <BOARD_ID> <FIELD_REF>
  --json

  Writes the field's default_value to every non-deleted card that does not
  already have a value set for this field. Returns the count of updated cards.
  JSON output:  {"field_id":"<uuid>","updated_count":<int>}

────────────────────────────────────────────────────────────────────────────────
## card field-set / field-get / field-clear / field-list

### card field-set <BOARD_ID> <CARD_ID> <FIELD_REF> <VALUE>
  --json

  Sets a custom field value on a card. Value is validated against the field type.
  FIELD_REF may be field name or UUID. CARD_ID may be UUID or card number (e.g. "a7f3-42").
  JSON output:  {"card_id":"<uuid>","field_id":"<uuid>","field_name":"<str>","value":"<str>"}

### card field-get <BOARD_ID> <CARD_ID> <FIELD_REF>
  --json

  Gets the current value of a custom field on a card.
  JSON output:  {"field_id":"<uuid>","field_name":"<str>","value":"<str>"|null}

### card field-clear <BOARD_ID> <CARD_ID> <FIELD_REF>
  --json

  Removes a custom field value from a card. Does not affect the field definition.
  JSON output:  {"cleared":"<field_uuid>","card_id":"<uuid>"}

### card field-list <BOARD_ID> <CARD_ID>
  --json

  Lists all custom field values set on a card with resolved field names.
  JSON output:  [{"field_id":"<uuid>","name":"<str>","value":"<str>"}, ...]

────────────────────────────────────────────────────────────────────────────────
## card create (with custom fields)

### card create <BOARD_ID> <COL_ID> <TITLE>
  --field FIELD_NAME_OR_UUID=VALUE   (repeat for multiple)
  --json

  Creates a card. If --field is supplied, those values are written first; then
  auto_apply defaults are applied for any remaining unset fields. Explicit
  --field values always beat auto-apply defaults.
  JSON output:  {"id":"<uuid>","title":"<str>","board_id":"<uuid>","number":<str|null>}

────────────────────────────────────────────────────────────────────────────────
## card list (with custom field filtering)

### card list <BOARD_ID>
  --col <COL_ID>            Filter to a specific column
  --label <LABEL>           Filter by label (exact match)
  --where FIELD_REF=VALUE   Filter by custom field (repeat for AND)
                            Operators: =  !=  >  >=  <  <=  ~(contains)
  --json

  --where expressions use AND semantics. Filters are evaluated via the SQLite
  custom field index (efficient even on large boards). FIELD_REF may be name or UUID.
  Example: monotaskcli card list $BOARD --where "Stage=Qualified" --where "Amount>10000"

────────────────────────────────────────────────────────────────────────────────
## card upsert (CRM upsert pattern)

### card upsert <BOARD_ID> <COL_ID> <TITLE>
  --match-field FIELD_REF   Field to match on when searching for an existing card
  --match-value VALUE       Value the match-field must equal
  --field FIELD=VALUE       Set field values (repeat for multiple)
  --json

  If a non-deleted card exists with match_field == match_value, its fields are
  updated. Otherwise a new card is created in COL_ID with the title and fields set.
  On create, auto_apply defaults are applied after explicit --field values.
  JSON output:  {"card_id":"<uuid>","created":<bool>,"board_id":"<uuid>"}

────────────────────────────────────────────────────────────────────────────────
## board schema

### board schema <BOARD_ID>
  --json

  Shows the board's columns and all active custom field definitions in one call.
  Useful as a first call to discover what fields and columns exist before operating.
  JSON output:  {"board_id":"<uuid>","title":"<str>","columns":[...],"fields":[...]}

### board undo <BOARD_ID>
  --json

  Restores the board to its state before the most recent mutation made by the
  local identity. Moves that entry to the redo stack. Returns false if nothing
  to undo.
  JSON output:  {"ok":true}  or  {"ok":false,"reason":"nothing to undo"}

### board redo <BOARD_ID>
  --json

  Re-applies the most recently undone mutation on the board. Returns false if
  nothing to redo.
  JSON output:  {"ok":true}  or  {"ok":false,"reason":"nothing to redo"}

────────────────────────────────────────────────────────────────────────────────
## checklist
Checklists are ordered task lists attached to a card. A card can have multiple
checklists, each with its own items.

### checklist add <BOARD_ID> <CARD_ID> <TITLE>
  --json

  Creates a new checklist on the card.
  JSON output:  {"id":"<uuid>","title":"<str>","items":[]}

### checklist item-add <BOARD_ID> <CARD_ID> <CHECKLIST_ID> <TEXT>
  --json

  Adds an unchecked item to a checklist.
  JSON output:  {"id":"<uuid>","text":"<str>","checked":false}

### checklist item-check <BOARD_ID> <CARD_ID> <CHECKLIST_ID> <ITEM_ID>
  --json

  Marks a checklist item as checked.
  JSON output:  {"checked":true,"item_id":"<uuid>"}

### checklist item-uncheck <BOARD_ID> <CARD_ID> <CHECKLIST_ID> <ITEM_ID>
  --json

  Marks a checklist item as unchecked.
  JSON output:  {"checked":false,"item_id":"<uuid>"}

### checklist item-delete <BOARD_ID> <CARD_ID> <CHECKLIST_ID> <ITEM_ID>
  --json

  Removes an item from a checklist permanently.

### checklist delete <BOARD_ID> <CARD_ID> <CHECKLIST_ID>
  --json

  Deletes an entire checklist and all its items from the card.

────────────────────────────────────────────────────────────────────────────────
## space
Spaces are shared containers that group boards and members. They enable
multi-user collaboration via signed invite tokens.

Space ownership: The creator is the owner (cannot be changed).
Members: Any user who joins via a valid invite token.
Boards: Boards are associated with a space; they can be on multiple spaces.

### space create <NAME>
  Creates a new space owned by the current user.
  Output:  "Created Space: <name> (<id>)"

### space list
  Lists all spaces stored locally.
  Output:  "<id> | <name> | <member_count> members"

### space info <SPACE_ID>
  Prints full details: name, owner pubkey, member list, board IDs.
  Members are shown as: "  <pubkey[0..16]>  <display_name>"

### space invite generate <SPACE_ID>
  Generates a new signed invite token (revokes previous tokens first).
  Output: the raw Base58 token string (share this with invitees)

  Token format:  Base58-encoded 120-byte payload
    Bytes 0-15:  space_id (raw UUID bytes)
    Bytes 16-47: owner Ed25519 pubkey (32 bytes)
    Bytes 48-55: creation timestamp (u64 big-endian unix ms)
    Bytes 56-119: Ed25519 signature over bytes 0-55

### space invite export <SPACE_ID> <OUTPUT_FILE>
  Generates an invite and writes a .space JSON file containing:
    {"token":"<base58>","space_name":"<str>","space_doc":"<base64-automerge>"}
  The .space file includes the full space CRDT document so the joiner gets
  the current member list and board references immediately.

### space invite revoke <SPACE_ID>
  Invalidates all active invite tokens for the space.
  Existing members are not affected; only new joins are blocked.

### space join <TOKEN_OR_FILE>
  Joins a space using either:
    - A raw Base58 token string
    - A path to a .space JSON file (recommended; includes space document)

  The command verifies the token signature, checks it hasn't been revoked,
  then adds the local user as a member of the space.
  Idempotent: safe to run again if already a member.
  Output: "Joined Space: <name> (<id>)"

### space boards add <SPACE_ID> <BOARD_ID>
  Associates a local board with the space.
  The board must already exist locally (created via `board create`).

### space boards remove <SPACE_ID> <BOARD_ID>
  Removes the board association from the space (board data is not deleted).

### space boards list <SPACE_ID>
  Prints one board ID per line for all boards in the space.

### space members list <SPACE_ID>
  Prints one member per line: "<pubkey>  <display_name>"
  Kicked members are shown with " [kicked]" suffix.

### space members kick <SPACE_ID> <PUBKEY>
  Marks a member as kicked in the space document and local DB.
  Kicked members cannot interact with the space (enforcement is app-level).

────────────────────────────────────────────────────────────────────────────────
## profile
Manages the local user's identity and display information.

### profile show
  Prints:
    Pubkey:       <64-char hex>
    Display name: <name> or "(not set)"
    Avatar:       "set" or "not set"
    SSH key path: <path> or "(auto-generated)"

### profile set-name <NAME>
  Sets your display name (shown to other space members).

### profile set-avatar <PATH>
  Reads an image file (any format) and stores it as your avatar blob.

### profile import-ssh-key [PATH]
  Imports an OpenSSH Ed25519 private key as your identity.
  If PATH is omitted, defaults to ~/.ssh/id_ed25519.
  WARNING: This changes your public key — space memberships tied to the old
           key will no longer match. Run this before joining any spaces.

────────────────────────────────────────────────────────────────────────────────
## github
Bidirectional sync with GitHub Issues. Requires a GitHub Personal Access Token.

### github connect [TOKEN]
  Saves a GitHub PAT (ghp_…) to local storage. Reads from stdin if omitted.
  The token needs repo scope (read/write issues and comments).

### github status
  Shows whether a GitHub token is saved locally. Does not print the token
  or make any network call. To verify connectivity, re-run `github connect`.

### github link <BOARD_ID> <OWNER> <REPO> --done-col <COL_ID>
  Links a board to a GitHub repository.
  OWNER: GitHub user or org name.
  REPO:  Repository name.
  --done-col: Column ID whose cards map to "closed" GitHub issues.

### github unlink <BOARD_ID>
  Removes the GitHub repository link from a board.

### github sync <BOARD_ID>
  Runs a full bidirectional sync between the board and linked GitHub repo:
  - Pulls new/updated GitHub issues → creates/updates cards
  - Pulls new GitHub comments → adds comments to cards (with avatar_url)
  - Pushes new local cards → creates GitHub issues
  - Pushes card moves to done column → closes GitHub issues

────────────────────────────────────────────────────────────────────────────────
## linear
Bidirectional sync with Linear issues. Requires a Linear API key.

### linear connect [TOKEN]
  Saves a Linear API key to local storage. Reads from stdin if omitted.

### linear status
  Shows token status and lists accessible Linear teams.

### linear teams
  Lists all teams accessible with the saved token (id and name).

### linear projects <TEAM_ID>
  Lists all projects for the given Linear team.

### linear link <BOARD_ID> --team <TEAM_ID> --project <PROJECT_ID> [--done-col <COL_ID>]
  Links a board to a Linear project. Creates Monotask columns matching
  the Linear workflow states automatically.
  --done-col: Optional column to map to "completed" Linear state.

### linear unlink <BOARD_ID>
  Removes the Linear project link from a board.

### linear sync <BOARD_ID>
  Runs a full bidirectional sync:
  - Pulls new/updated Linear issues → creates/updates cards
  - Pulls new Linear comments → adds comments to cards
  - Pushes new local cards → creates Linear issues
  - Pushes card state changes → updates Linear issue state

────────────────────────────────────────────────────────────────────────────────
## mail
Gmail and Outlook email integration. Syncs email contacts into boards as cards.
One card per unique sender. Recent emails stored as comments. BYO OAuth2 credentials.

### mail gmail-connect --client-id <CLIENT_ID>
  Connects Gmail via OAuth2 PKCE. Opens your browser for authorization.
  Get a client ID: Google Cloud Console → APIs & Services → Credentials → OAuth 2.0 Client ID (Desktop app).
  Enable the Gmail API in your project first.

### mail outlook-connect --client-id <CLIENT_ID> [--tenant-id <TENANT>]
  Connects Outlook via OAuth2 PKCE. Opens your browser for authorization.
  Get a client ID: Azure Portal → App Registrations → New registration (Mobile/Desktop app).
  Tenant defaults to "common" (personal + work accounts).

### mail status
  Shows connection status for Gmail and Outlook.

### mail disconnect <PROVIDER>
  Removes saved credentials for "gmail" or "outlook".

### mail link <BOARD_ID> [--provider both|gmail|outlook] [--inbox-col <COL_ID>] [--keep-last <N>]
  Links a board to receive email contacts. New contacts go to the inbox column.
  --provider: which provider(s) to sync (default: both)
  --inbox-col: column ID for new contact cards (default: first column)
  --keep-last: number of recent emails to keep as comments per contact (default: 2)
  Set MAIL_GMAIL_CLIENT_ID and MAIL_OUTLOOK_CLIENT_ID env vars for sync.

### mail unlink <BOARD_ID>
  Removes the email sync link from a board.

### mail sync <BOARD_ID>
  Fetches emails since last sync (or last 30 days on first run), groups by sender,
  and upserts one card per contact with recent emails as comments.
  Custom fields updated: Email, Last Seen, Email Count, Provider, Labels.

────────────────────────────────────────────────────────────────────────────────
## chat
Per-space persistent P2P chat backed by Automerge CRDT. Messages sync via the
same pipeline as boards. Each space has one chat doc keyed `{space_id}-chat`.

### chat send <SPACE_ID> <TEXT>
  --json

  Appends a message to the space chat. Author is set to the local identity pubkey.
  JSON output:  {"id":"<uuid>","author":"<pubkey-hex>","text":"<str>","created_at":<unix>,"refs":[]}

### chat list <SPACE_ID>
  --limit <N>   Maximum messages to return (default: 50)
  --json

  Lists recent chat messages in chronological order (oldest first).
  JSON output:  [{"id":"...","author":"...","text":"...","created_at":...,"refs":[...]}, ...]

────────────────────────────────────────────────────────────────────────────────
## app
Commands for interacting with the Monotask desktop application.

### app open <URL>
  Opens a monotask:// deep link in the running desktop app.
  Supported URL patterns:
    monotask://board/<BOARD_ID>             — navigate to a board
    monotask://board/<BOARD_ID>/card/<ID>  — open a specific card

  Example:
    monotaskcli app open monotask://board/a1b2c3d4-...
    monotaskcli app open "monotask://board/abc/card/xyz"

────────────────────────────────────────────────────────────────────────────────
## sync
Starts the P2P sync daemon using iroh QUIC transport (direct address + bootstrap peers).
The daemon keeps boards in sync with other Monotask peers on the local network.

### sync [OPTIONS]
  --detach           Run in background (writes PID to data dir)
  --stop             Stop the running background daemon
  --status           Show sync status (running / stopped)
  --port <PORT>      TCP port to listen on (default: OS-assigned)
                     Use a fixed port when peers need to dial you directly.
  --peer <MULTIADDR> Dial a specific peer at startup, bypassing mDNS discovery.
                     Format: /ip4/1.2.3.4/tcp/7272
                     Repeat the flag to add multiple peers.

  Example (background daemon with fixed port):
    monotaskcli sync --detach --port 7272

  Example (connect directly to a known peer):
    monotaskcli sync --peer /ip4/192.168.1.10/tcp/7272

--------------------------------------------------------------------------------
TIMESTAMPS (HLC FORMAT)
--------------------------------------------------------------------------------
All created_at / timestamp fields use Hybrid Logical Clock format:
  "<wall_ms_hex>-<logical_hex>"
  Example: "018f3a2b4c5d6e7f-00000001"
           wall_ms  = 018f3a2b4c5d6e7f (hex, Unix milliseconds)
           logical  = 00000001 (hex counter, increments on same-ms operations)

To convert to a Unix timestamp in milliseconds:
  ms = parseInt(hlc.split('-')[0], 16)
To convert to a human date (JavaScript):
  new Date(ms).toISOString()

--------------------------------------------------------------------------------
ID FORMATS
--------------------------------------------------------------------------------
Board ID      : UUID v4, e.g. "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
Column ID     : UUID v4
Card ID       : UUID v4  ← use this in all card/comment/checklist commands
Card Number   : "<base32-prefix>-<seq>" e.g. "a7f3-42"  (human-readable only;
                CLI commands require the full UUID card ID, not the number)
Space ID      : UUID v4
Comment ID    : UUID v4
Checklist ID  : UUID v4
Item ID       : UUID v4
Attachment ID : 6-char hex string (referenced as "img:<id>" in markdown)
Field ID      : UUID v4  ← CLI also accepts field name in all field commands

--------------------------------------------------------------------------------
STORAGE
--------------------------------------------------------------------------------
Location:  ~/.local/share/monotaskcli/monotaskcli.db  (or custom --data-dir)

Database tables:
  boards            board_id | automerge_doc (BLOB) | last_modified | last_heads
  card_number_index board_id | card_id | number
  spaces            id | name | owner_pubkey | created_at | automerge_bytes
  space_members     space_id | pubkey | display_name | avatar_blob | kicked
  space_boards      space_id | board_id
  space_invites     token_hash (PK) | token | space_id | created_at | revoked
  user_profile      pk='local' | pubkey | display_name | avatar_blob | ssh_key_path

Board data is stored as Automerge CRDT binary documents. The root map contains:
  columns            – list of column objects [{id, title, card_ids[]}]
  cards              – map of card_id → card object (each card has a custom_fields sub-map)
  members            – map of pubkey → member profile
  actor_card_seq     – map of pubkey → int (per-actor card counter)
  label_definitions  – map of label_id → label object
  field_definitions  – map of field_id → FieldDefinition object (custom fields)

SQLite index:
  card_custom_field_index  board_id | card_id | field_id | value_text | value_num | value_date

--------------------------------------------------------------------------------
COMMON AGENT WORKFLOWS
--------------------------------------------------------------------------------

### Workflow: Create a board and populate it
  SPACE=$(monotaskcli space list | awk 'NR==1{print $1}')   # pick first space
  BOARD=$(monotaskcli board create "My Board" --space $SPACE --json | jq -r .id)
  TODO_COL=$(monotaskcli column create $BOARD "Todo" --json | jq -r .id)
  DOING_COL=$(monotaskcli column create $BOARD "Doing" --json | jq -r .id)
  DONE_COL=$(monotaskcli column create $BOARD "Done" --json | jq -r .id)
  CARD=$(monotaskcli card create $BOARD $TODO_COL "First task" --json | jq -r .id)
  monotaskcli card view $BOARD $CARD --json

### Workflow: Inspect all cards in a board
  COLS=$(monotaskcli column list $BOARD --json)
  echo $COLS | jq -r '.[].card_ids[]' | while read CARD_ID; do
    monotaskcli card view $BOARD $CARD_ID --json
  done

### Workflow: Score and prioritise cards
  # ICE scoring: set impact (value) and effort (cost), priority auto-computed
  monotaskcli card set-impact  $BOARD $CARD 8
  monotaskcli card set-effort  $BOARD $CARD 3
  # Or set priority directly without impact/effort
  monotaskcli card set-direct-priority $BOARD $CARD 9
  # Reset all scoring fields
  monotaskcli card clear-priority $BOARD $CARD

### Workflow: Attach an image and reference it in the description
  ATT=$(monotaskcli card attach-image $BOARD $CARD screenshot.png --json | jq -r .token)
  monotaskcli card set-description $BOARD $CARD "See: ![$ATT]($ATT)"

### Workflow: Build a subtask hierarchy
  PARENT=$(monotaskcli card create $BOARD $TODO_COL "Epic: Auth" --json | jq -r .id)
  monotaskcli card subtask add $BOARD $PARENT $BOARD $TODO_COL "Subtask: Login page" --json
  monotaskcli card subtask list $BOARD $PARENT --json

### Workflow: Link GitHub and sync
  monotaskcli github connect ghp_yourtoken
  monotaskcli github link $BOARD myorg myrepo --done-col $DONE_COL
  monotaskcli github sync $BOARD

### Workflow: Link Linear and sync
  monotaskcli linear connect lin_api_yourkey
  monotaskcli linear teams
  monotaskcli linear projects <TEAM_ID>
  monotaskcli linear link $BOARD --team <TEAM_ID> --project <PROJECT_ID>
  monotaskcli linear sync $BOARD

### Workflow: Collaborative space setup (two users, A and B)
  # --- User A ---
  SPACE=$(monotaskcli space create "Team" | awk '{print $NF}' | tr -d '()')
  monotaskcli space boards add $SPACE $BOARD
  monotaskcli space invite export $SPACE invite.space
  # Share invite.space with User B

  # --- User B ---
  monotaskcli space join invite.space
  monotaskcli space boards list $SPACE   # see boards shared by A

### Workflow: Build a CRM pipeline with custom fields
  # 1. Create the board and columns
  SPACE=$(monotaskcli space list | awk 'NR==1{print $1}')
  BOARD=$(monotaskcli board create "CRM Pipeline" --space $SPACE --json | jq -r .id)
  NEW=$(monotaskcli column create $BOARD "New Leads" --json | jq -r .id)
  QUAL=$(monotaskcli column create $BOARD "Qualified" --json | jq -r .id)
  WON=$(monotaskcli column create $BOARD "Won" --json | jq -r .id)

  # 2. Define custom fields
  STAGE=$(monotaskcli field create $BOARD "Stage" --field-type select \
    --option "Lead" --option "Qualified" --option "Won" \
    --default-value "Lead" --auto-apply --json | jq -r .id)
  monotaskcli field create $BOARD "Company" --field-type text --json
  monotaskcli field create $BOARD "Amount" --field-type number --json
  monotaskcli field create $BOARD "Close Date" --field-type date --json

  # 3. Add leads with field values
  monotaskcli card create $BOARD $NEW "Acme Corp" \
    --field "Company=Acme Corp" --field "Amount=25000" --json

  # 4. Upsert a lead (create or update based on Company)
  monotaskcli card upsert $BOARD $NEW "Globex" \
    --match-field Company --match-value "Globex" \
    --field "Amount=50000" --field "Stage=Qualified" --json

  # 5. Query qualified leads with amount > 20000
  monotaskcli card list $BOARD --where "Stage=Qualified" --where "Amount>20000" --json

  # 6. Show full schema
  monotaskcli board schema $BOARD --json

### Workflow: Add a checklist to a card
  CL=$(monotaskcli checklist add $BOARD $CARD "Definition of Done" --json | jq -r .id)
  ITEM=$(monotaskcli checklist item-add $BOARD $CARD $CL "Write tests" --json | jq -r .id)
  monotaskcli checklist item-check $BOARD $CARD $CL $ITEM

### Workflow: Comment thread
  monotaskcli card comment add $BOARD $CARD "Starting work on this"
  monotaskcli card comment add $BOARD $CARD "Blocked on API access"
  monotaskcli card comment list $BOARD $CARD --json

--------------------------------------------------------------------------------
ERROR HANDLING
--------------------------------------------------------------------------------
All commands exit with code 0 on success, non-zero on error.
Errors are printed to stderr as plain text (not JSON).
Common error causes:
  - Board/card/column/space ID not found in local database
  - Invalid UUID format for IDs
  - Board file corrupted or missing
  - Invite token invalid signature or revoked
  - SSH key file not found or wrong format (must be Ed25519)

--------------------------------------------------------------------------------
LIMITATIONS & NOTES FOR AGENTS
--------------------------------------------------------------------------------
- The CLI does NOT sync between users automatically. P2P sync is handled
  by the desktop app (Monotask GUI). The CLI operates only on local data.
- `card create` currently uses a placeholder identity for card numbers
  (all cards get prefix "aaaa"). Full identity wiring is planned.
- To get a card's current column before moving: iterate `column list` and check
  which column's card_ids contains the card UUID.
- `card view --json` returns the full Card struct; the `number` field is a
  JSON object {"prefix":"...","seq":N}, not the display string "prefix-N".
- Invite tokens are single-use per generation: generating a new token revokes
  the previous one. Use `invite export` (not `invite generate`) to share
  invites that include full space state.
- Data directory must be consistent across all CLI invocations for the same
  instance. If using --data-dir, always pass the same path.
- Custom field definitions are stored per-board in the Automerge document.
  Use `board schema` as a first call to discover columns + fields before writing.
- `card field-set` validates the value against the field type before writing.
  Number fields must be parseable as f64; Date fields must be YYYY-MM-DD;
  Select fields must match one of the declared options.
- `card upsert` does a linear scan to match cards by field value. For large boards,
  prefer `card list --where` (uses SQLite index) to locate IDs first.
- `field backfill` only sets the field on cards that currently have no value for it.
  Cards with existing values (even if different from the default) are left unchanged.

================================================================================
"##);
}

/// Parse a list of "KEY=VALUE" strings from --field flags.
/// KEY is a field name or UUID; VALUE is the string to store.
fn parse_field_assignments(pairs: &[String]) -> anyhow::Result<Vec<(String, String)>> {
    let mut result = Vec::new();
    for pair in pairs {
        let pos = pair.find('=').ok_or_else(|| anyhow::anyhow!(
            "invalid --field '{}': expected FIELD_NAME=VALUE", pair
        ))?;
        let key = pair[..pos].trim().to_string();
        let value = pair[pos + 1..].to_string();
        if key.is_empty() {
            return Err(anyhow::anyhow!("invalid --field '{}': field name cannot be empty", pair));
        }
        result.push((key, value));
    }
    Ok(result)
}

/// Parse a --where expression like "Stage=Qualified", "Amount>10000", "Name~Acme".
/// Returns (field_ref, operator, value).
fn parse_filter_expr(expr: &str) -> Option<(String, String, String)> {
    for op in &[">=", "<=", "!=", ">", "<", "~", "="] {
        if let Some(pos) = expr.find(op) {
            let field_ref = expr[..pos].trim().to_string();
            let value = expr[pos + op.len()..].trim().to_string();
            if !field_ref.is_empty() {
                return Some((field_ref, op.to_string(), value));
            }
        }
    }
    None
}

fn parse_token_or_file(input: &str) -> anyhow::Result<(String, String, Option<Vec<u8>>)> {
    if input.ends_with(".space") || std::path::Path::new(input).exists() {
        let content = std::fs::read_to_string(input)?;
        let v: serde_json::Value = serde_json::from_str(&content)?;
        let token = v["token"].as_str()
            .ok_or_else(|| anyhow::anyhow!("missing or invalid 'token' field in .space file"))?
            .to_string();
        let name = v["space_name"].as_str().unwrap_or("Shared Space").to_string();
        let doc_b64 = v["space_doc"].as_str().unwrap_or("");
        let doc_bytes = if doc_b64.is_empty() {
            None
        } else {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.decode(doc_b64).ok()
        };
        Ok((token, name, doc_bytes))
    } else {
        // Bare token — space_doc and name come from the embedded payload (v2 token)
        Ok((input.to_string(), String::new(), None))
    }
}

fn mime_from_ext(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "png"  => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif"  => "image/gif",
        "webp" => "image/webp",
        "svg"  => "image/svg+xml",
        _      => "image/png",
    }
}
