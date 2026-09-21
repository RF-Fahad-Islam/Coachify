use crate::db::{self, DbError};
use crate::phone;
use crate::storage;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::RwLock;
use std::time::Instant;

pub const SYSTEM_CLI_USER_ID: &str = "00000000-0000-0000-0000-000000000000";

static STATUS_STAGE: RwLock<String> = RwLock::new(String::new());
static STATUS_PERCENT: AtomicU8 = AtomicU8::new(0);
static STATUS_STATE: RwLock<String> = RwLock::new(String::new());

fn update_status(state: &str, stage: &str, percent: u8) {
    if let Ok(mut s) = STATUS_STATE.write() {
        *s = state.to_string();
    }
    if let Ok(mut st) = STATUS_STAGE.write() {
        *st = stage.to_string();
    }
    STATUS_PERCENT.store(percent, Ordering::Relaxed);
}

pub fn get_current_status() -> GeneratorStatusResponse {
    let state = STATUS_STATE
        .read()
        .map(|s| s.clone())
        .unwrap_or_else(|_| "idle".to_string());
    let stage = STATUS_STAGE.read().map(|s| s.clone()).unwrap_or_default();
    let percent = STATUS_PERCENT.load(Ordering::Relaxed);
    let is_demo_active = db::is_demo_database_active();
    let active_path = db::resolve_active_db_path()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    GeneratorStatusResponse {
        status: if state.is_empty() {
            "idle".to_string()
        } else {
            state
        },
        percent,
        stage,
        is_demo_active,
        active_database_path: active_path,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum DemoProfile {
    Demo,
    Load,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GenerateDatasetPayload {
    pub profile: DemoProfile,
    pub seed: Option<u64>,
    pub out_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GenerationSummary {
    pub profile: String,
    pub seed: u64,
    pub student_count: u32,
    pub guardian_count: u32,
    pub batch_count: u32,
    pub charge_count: u32,
    pub payment_count: u32,
    pub db_path: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GeneratorStatusResponse {
    pub status: String,
    pub percent: u8,
    pub stage: String,
    pub is_demo_active: bool,
    pub active_database_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SwitchDatabasePayload {
    pub target_filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SwitchDatabaseResponse {
    pub success: bool,
    pub active_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SwitchToProductionResponse {
    pub success: bool,
    pub production_path: String,
}

// -----------------------------------------------------------------------------
// Embedded Bilingual Demographics Dictionaries
// -----------------------------------------------------------------------------

pub const MALE_FIRST_NAMES: &[(&str, &str)] = &[
    ("Rahim", "রহিম"),
    ("Karim", "করিম"),
    ("Tanvir", "তানভীর"),
    ("Sakib", "সাকিব"),
    ("Faisal", "ফয়সাল"),
    ("Hasan", "হাসান"),
    ("Tamim", "তামিম"),
    ("Mahmud", "মাহমুদ"),
    ("Naim", "নাঈম"),
    ("Arafat", "আরাফাত"),
    ("Sabbir", "সাব্বির"),
    ("Rifat", "রিফাত"),
    ("Zubair", "জুবায়ের"),
    ("Imran", "ইমরান"),
    ("Arif", "আরিফ"),
    ("Shakil", "শাকিল"),
    ("Mehedi", "মেহেদী"),
    ("Rakib", "রাকিব"),
    ("Alamin", "আল-আমিন"),
    ("Sohag", "সোহাগ"),
    ("Ashik", "আশিক"),
    ("Tareq", "তারেক"),
    ("Nahid", "নাহিদ"),
    ("Farhan", "ফারহান"),
    ("Robiul", "রবিউল"),
    ("Shahadat", "শাহাদাত"),
    ("Jahid", "জাহিদ"),
    ("Mustafiz", "মুস্তাফিজ"),
    ("Soumya", "সৌম্য"),
    ("Taskin", "তাসকিন"),
    ("Anis", "আনিস"),
    ("Habib", "হাবিব"),
    ("Kamal", "কামাল"),
    ("Nasir", "নাসির"),
    ("Rubel", "রুবেল"),
    ("Saiful", "সাইফুল"),
    ("Tariqul", "তরিকুল"),
    ("Zahid", "জাহিদ"),
    ("Kawsar", "কাউসার"),
    ("Shahid", "শহীদ"),
];

pub const FEMALE_FIRST_NAMES: &[(&str, &str)] = &[
    ("Sadia", "সাদিয়া"),
    ("Farhana", "ফারহানা"),
    ("Nusrat", "নুসরাত"),
    ("Anika", "আনিকা"),
    ("Sumaiya", "সুমাইয়া"),
    ("Ayesha", "আয়েশা"),
    ("Fatema", "ফাতেমা"),
    ("Mim", "মীম"),
    ("Jannat", "জান্নাত"),
    ("Samia", "সামিয়া"),
    ("Tasnim", "তাসনিম"),
    ("Sharmin", "শারমিন"),
    ("Ritu", "রিতু"),
    ("Tamanna", "তামান্না"),
    ("Sabrina", "সাবরিনা"),
    ("Bristy", "বৃষ্টি"),
    ("Mitu", "মিতু"),
    ("Nadia", "নাদিয়া"),
    ("Tanjila", "তানজিলা"),
    ("Sultana", "সুলতানা"),
    ("Fahmida", "ফাহমিদা"),
    ("Mahfuza", "মাহফুজা"),
    ("Rumpa", "রুম্পা"),
    ("Shampa", "শম্পা"),
    ("Lamia", "লামিয়া"),
    ("Nafisa", "নাফিসা"),
    ("Rumana", "রুমানা"),
    ("Priya", "প্রিয়া"),
    ("Marufa", "মারুফা"),
    ("Ishrat", "ইশরাত"),
    ("Meher", "মেহের"),
    ("Salma", "সালমা"),
    ("Kohinur", "কোহিনুর"),
    ("Bilkis", "বিলকিস"),
    ("Shirina", "শিরিনা"),
    ("Rokeya", "রোকেয়া"),
    ("Shamima", "শামীমা"),
    ("Tahmina", "তাহমিনা"),
    ("Jannatul", "জান্নাতুল"),
    ("Shirin", "শিরিন"),
];

pub const FAMILY_SURNAMES: &[(&str, &str)] = &[
    ("Chowdhury", "চৌধুরী"),
    ("Ahmed", "আহমেদ"),
    ("Khan", "খাঁন"),
    ("Rahman", "রহমান"),
    ("Islam", "ইসলাম"),
    ("Hossain", "হোসেন"),
    ("Haque", "হক"),
    ("Ali", "আলী"),
    ("Sikder", "সিকদার"),
    ("Miah", "মিয়া"),
    ("Bhuiyan", "ভূঁইয়া"),
    ("Talukder", "তালুকদার"),
    ("Kazi", "কাজী"),
    ("Majumder", "মজুমদার"),
    ("Dewan", "দেওয়ান"),
    ("Gazi", "গাজী"),
    ("Howlader", "হাওলাদার"),
    ("Mondal", "মণ্ডল"),
    ("Sarder", "সরদার"),
    ("Bakshi", "বক্সী"),
    ("Molla", "মোল্লা"),
    ("Khandaker", "খন্দকার"),
    ("Akter", "আক্তার"),
    ("Begum", "বেগম"),
    ("Sarkar", "সরকার"),
];

pub const DHAKA_LOCATIONS: &[&str] = &[
    "Farmgate, Dhaka",
    "Mirpur-10, Dhaka",
    "Mirpur-2, Dhaka",
    "Dhanmondi 27, Dhaka",
    "Dhanmondi 32, Dhaka",
    "Uttara Sector 4, Dhaka",
    "Uttara Sector 11, Dhaka",
    "Mohakhali, Dhaka",
    "Banani, Dhaka",
    "Gulshan-1, Dhaka",
    "Mohammadpur, Dhaka",
    "Badda, Dhaka",
    "Malibagh, Dhaka",
    "Shantinagar, Dhaka",
    "Lalbagh, Dhaka",
    "Azimpur, Dhaka",
    "Rampura, Dhaka",
    "Khilgaon, Dhaka",
    "Bashundhara R/A, Dhaka",
    "Panthapath, Dhaka",
];

pub const MOBILE_PREFIXES: &[&str] = &["017", "018", "019", "013", "016", "015"];

// -----------------------------------------------------------------------------
// Path Sandboxing & Cleanup Guard
// -----------------------------------------------------------------------------

pub fn resolve_demo_dir() -> PathBuf {
    if let Ok(Some(cfg)) = crate::storage::load_global_config() {
        PathBuf::from(&cfg.data_dir).join("demo")
    } else {
        let base = match std::env::var("LOCALAPPDATA") {
            Ok(val) => PathBuf::from(val),
            Err(_) => {
                let home = std::env::var("USERPROFILE")
                    .or_else(|_| std::env::var("HOME"))
                    .unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".coachify")
            }
        };
        base.join("Coachify").join("demo")
    }
}

struct TempFileCleanupGuard {
    path: PathBuf,
    disarmed: bool,
}

impl TempFileCleanupGuard {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            disarmed: false,
        }
    }

    fn disarm(&mut self) {
        self.disarmed = true;
    }
}

impl Drop for TempFileCleanupGuard {
    fn drop(&mut self) {
        if !self.disarmed && self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

// -----------------------------------------------------------------------------
// Core Dataset Generator
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct DatasetGenerator {
    profile: DemoProfile,
    seed: u64,
    target_path: PathBuf,
    tmp_path: PathBuf,
    rng: ChaCha8Rng,
}

impl DatasetGenerator {
    pub fn new(
        profile: DemoProfile,
        seed: Option<u64>,
        out_filename: Option<String>,
    ) -> Result<Self, DbError> {
        let effective_seed = match seed {
            Some(s) => s,
            None => match profile {
                DemoProfile::Load => 42,
                DemoProfile::Demo => rand::random::<u64>(),
            },
        };

        let demo_dir = resolve_demo_dir();
        fs::create_dir_all(&demo_dir)?;

        let filename = match out_filename {
            Some(ref name) => {
                // AC-9: Path sandboxing against directory traversal
                if name.contains("..") || name.contains('/') || name.contains('\\') {
                    return Err(DbError::Config(
                        "Target filename contains illegal path traversal characters".to_string(),
                    ));
                }
                name.clone()
            }
            None => match profile {
                DemoProfile::Demo => "demo_500.db".to_string(),
                DemoProfile::Load => "load_10k.db".to_string(),
            },
        };

        let target_path = demo_dir.join(&filename);

        // AC-9: Guard against overwriting active production database
        let prod_path = db::resolve_dev_db_path()?;
        if filename == "coachify.db" || target_path == prod_path {
            return Err(DbError::Config(
                "Target path matches active production database. Aborting to protect live records."
                    .to_string(),
            ));
        }
        if let (Ok(can_target), Ok(can_prod)) =
            (target_path.canonicalize(), prod_path.canonicalize())
        {
            if can_target == can_prod {
                return Err(DbError::Config(
                    "Target path matches active production database. Aborting to protect live records."
                        .to_string(),
                ));
            }
        }

        // AC-8: Pre-flight disk space check
        let required_free_bytes: u64 = match profile {
            DemoProfile::Demo => 100 * 1024 * 1024,  // 100 MB
            DemoProfile::Load => 1024 * 1024 * 1024, // 1 GB
        };

        let (free_bytes, _) = storage::get_path_disk_space(&demo_dir);
        if free_bytes > 0 && free_bytes < required_free_bytes {
            return Err(DbError::Config(format!(
                "Insufficient disk space on target drive. Required: {} MB, available: {} MB",
                required_free_bytes / (1024 * 1024),
                free_bytes / (1024 * 1024)
            )));
        }

        let tmp_path = demo_dir.join(format!("{}.tmp", filename));
        if tmp_path.exists() {
            let _ = fs::remove_file(&tmp_path);
        }

        let rng = ChaCha8Rng::seed_from_u64(effective_seed);

        Ok(Self {
            profile,
            seed: effective_seed,
            target_path,
            tmp_path,
            rng,
        })
    }

    pub fn generate(&mut self) -> Result<GenerationSummary, DbError> {
        let start_time = Instant::now();
        let mut cleanup_guard = TempFileCleanupGuard::new(self.tmp_path.clone());

        update_status("generating", "Initializing Schema", 5);

        // Open temp connection with bulk PRAGMAs (AC-7)
        let conn = Connection::open(&self.tmp_path)?;
        conn.pragma_update(None, "synchronous", "OFF")?;
        conn.pragma_update(None, "journal_mode", "OFF")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        // Apply all baseline migrations
        db::apply_all_migrations(&conn)?;

        // Create synthetic demo_metadata table (AC-10)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS demo_metadata (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                profile TEXT NOT NULL,
                seed INTEGER NOT NULL,
                student_count INTEGER NOT NULL,
                generated_at TEXT NOT NULL,
                generator_version TEXT NOT NULL
            )",
            [],
        )?;

        update_status(
            "generating",
            "Creating Institution & Academic Structure",
            15,
        );

        // Seed Institution Profile
        let now_utc = db::now_utc_iso();
        conn.execute(
            "INSERT OR REPLACE INTO institution (
                id, name, name_bn, address, email, phones, established_year, created_at, updated_at
            ) VALUES (
                1, 'Apex Academic Coaching Centre', 'এপেক্স একাডেমিক কোচিং সেন্টার',
                'Farmgate, Dhaka-1215', 'info@apexcoaching.demo', '8801700000000', 2018, ?1, ?1
            )",
            params![now_utc],
        )?;

        // Seed Academic Sessions (AC-3)
        conn.execute(
            "INSERT OR IGNORE INTO academic_sessions (id, name, start_date, end_date, is_current, created_at, updated_at)
             VALUES ('sess-2025', '2025 Academic Session', '2025-01-01', '2025-12-31', 0, ?1, ?1)",
            params![now_utc],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO academic_sessions (id, name, start_date, end_date, is_current, created_at, updated_at)
             VALUES ('sess-2026', '2026 Academic Session', '2026-01-01', '2026-12-31', 1, ?1, ?1)",
            params![now_utc],
        )?;

        // Seed Staff / Teachers
        let staff_ids = self.seed_staff(&conn, &now_utc)?;

        // Seed Subjects
        let subject_ids = self.seed_subjects(&conn, &now_utc)?;

        // Seed Programs & Batches (AC-3)
        let (programs, batches) =
            self.seed_programs_and_batches(&conn, &staff_ids, &subject_ids, &now_utc)?;

        // Seed Financial Fee Heads & Fee Plans (AC-4)
        let tuition_fee_head_id = "fh-tuition-demo";
        conn.execute(
            "INSERT OR IGNORE INTO fee_heads (id, name, name_bn, kind, created_at, updated_at)
             VALUES (?1, 'Monthly Tuition Fee', 'মাসিক টিউশন ফি', 'tuition', ?2, ?2)",
            params![tuition_fee_head_id, now_utc],
        )?;

        for prog_id in &programs {
            let plan_id = format!("fp-{}", prog_id);
            let amount_poisha: i64 = match prog_id.as_str() {
                "prog-hsc-sci" => 350000,
                "prog-hsc-com" => 300000,
                "prog-ssc-gen" => 250000,
                _ => 200000,
            };
            conn.execute(
                "INSERT OR IGNORE INTO fee_plans (
                    id, program_id, fee_head_id, amount_poisha, frequency, effective_from, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, 'monthly', '2025-01-01', ?5, ?5)",
                params![plan_id, prog_id, tuition_fee_head_id, amount_poisha, now_utc],
            )?;
        }

        update_status("generating", "Generating Students & Sibling Groups", 35);

        // Determine student population sizes (AC-5)
        let (active_student_count, archived_student_count) = match self.profile {
            DemoProfile::Demo => (500, 0),
            DemoProfile::Load => (10000, 15000),
        };
        let total_student_count = active_student_count + archived_student_count;

        // Generate Guardians and Students with ~20% Sibling Groups (AC-2)
        let (student_ids, guardian_count) = self.generate_students_and_guardians(
            &conn,
            active_student_count,
            archived_student_count,
            &now_utc,
        )?;

        update_status("generating", "Enrolling Students into Batches", 55);

        // Enroll students into Batches (AC-3)
        let enrollments = self.enroll_students(
            &conn,
            &student_ids[..active_student_count as usize],
            &batches,
            &now_utc,
        )?;

        update_status("generating", "Simulating Financial Ledger & Payments", 70);

        // Simulate Monthly Financial Ledger (AC-4)
        let (charges_count, payments_count) =
            self.simulate_financial_ledger(&conn, &enrollments, tuition_fee_head_id, &now_utc)?;

        update_status("generating", "Finalizing Database & Checkpointing WAL", 90);

        // Record metadata (AC-10)
        let profile_str = match self.profile {
            DemoProfile::Demo => "demo_500",
            DemoProfile::Load => "load_10k",
        };
        conn.execute(
            "INSERT OR REPLACE INTO demo_metadata (
                id, profile, seed, student_count, generated_at, generator_version
            ) VALUES (1, ?1, ?2, ?3, ?4, ?5)",
            params![
                profile_str,
                self.seed as i64,
                total_student_count as i64,
                now_utc,
                env!("CARGO_PKG_VERSION")
            ],
        )?;

        // AC-12: Record audit log in dataset
        let cli_session = crate::accounts::ResolvedSession {
            session_id: "cli-session".to_string(),
            user: crate::accounts::SessionUser {
                id: SYSTEM_CLI_USER_ID.to_string(),
                username: "SYSTEM_CLI".to_string(),
                display_name: "System CLI Operator".to_string(),
                status: "active".to_string(),
            },
            roles: vec!["Owner".to_string()],
            permissions: vec![],
            device: crate::accounts::DeviceSummary {
                id: "cli-device".to_string(),
                name: "CLI Console".to_string(),
                windows_name: "CLI".to_string(),
                is_host: true,
            },
            created_at: now_utc.clone(),
            last_seen_at: now_utc.clone(),
            expires_at: now_utc.clone(),
            absolute_expires_at: now_utc.clone(),
            is_idle_locked: false,
            is_expired: false,
            is_revoked: false,
        };
        let audit_entry = crate::audit::AuditEntry {
            action: "generator.generate_dataset".to_string(),
            entity: "demo_dataset".to_string(),
            entity_id: profile_str.to_string(),
            before_json: None,
            after_json: Some(serde_json::json!({
                "profile": profile_str,
                "seed": self.seed,
                "student_count": total_student_count,
            })),
        };
        let _ = crate::audit::record_audit_entry(&conn, audit_entry, Some(&cli_session), &now_utc);

        // Integrity verification (AC-7)
        db::verify_integrity(&conn)?;

        // Transition to WAL mode and truncate checkpoint (AC-7)
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let _ = conn.execute("PRAGMA wal_checkpoint(TRUNCATE);", []);

        drop(conn);

        // Atomic rename from .tmp to target path (AC-8)
        if self.target_path.exists() {
            let _ = fs::remove_file(&self.target_path);
        }
        fs::rename(&self.tmp_path, &self.target_path)?;
        cleanup_guard.disarm();

        let elapsed = start_time.elapsed().as_millis() as u64;
        update_status("completed", "Generation Complete", 100);

        Ok(GenerationSummary {
            profile: profile_str.to_string(),
            seed: self.seed,
            student_count: total_student_count,
            guardian_count,
            batch_count: batches.len() as u32,
            charge_count: charges_count,
            payment_count: payments_count,
            db_path: self.target_path.display().to_string(),
            duration_ms: elapsed,
        })
    }

    fn seed_staff(&mut self, conn: &Connection, now_utc: &str) -> Result<Vec<String>, DbError> {
        let staff_data = [
            ("Tanvir Ahmed", "তানভীর আহমেদ", "Senior Lecturer", "Physics"),
            ("Mohammad Ali", "মোহাম্মদ আলী", "Head Teacher", "Mathematics"),
            ("Sadia Sultana", "সাদিয়া সুলতানা", "Lecturer", "Chemistry"),
            ("Mahmudul Hasan", "মাহমুদুল হাসান", "Senior Teacher", "Biology"),
            ("Farhana Begum", "ফারহানা বেগম", "Lecturer", "English"),
            ("Kazi Nazrul", "কাজী নজরুল", "Assistant Teacher", "Bangla"),
            ("Abdur Rahman", "আব্দুর রহমান", "Lecturer", "Accounting"),
            (
                "Anika Tabassum",
                "আনিকা তাবাসসুম",
                "Lecturer",
                "Business Studies",
            ),
        ];

        let mut ids = Vec::new();
        for (i, (name, name_bn, designation, dept)) in staff_data.iter().enumerate() {
            let id = format!("staff-{:03}", i + 1);
            let phone = format!("88017{:08}", 11000000 + i);
            conn.execute(
                "INSERT INTO staff (
                    id, name, name_bn, phone_norm, designation, department, joining_date, status, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '2024-01-01', 'active', ?7, ?7)",
                params![id, name, name_bn, phone, designation, dept, now_utc],
            )?;
            ids.push(id);
        }
        Ok(ids)
    }

    fn seed_subjects(&mut self, conn: &Connection, now_utc: &str) -> Result<Vec<String>, DbError> {
        let subjects = [
            ("sub-phy", "Physics", "পদার্থবিজ্ঞান", "PHY-101"),
            ("sub-che", "Chemistry", "রসায়ন", "CHE-101"),
            ("sub-mth", "Higher Mathematics", "উচ্চতর গণিত", "MTH-101"),
            ("sub-bio", "Biology", "জীববিজ্ঞান", "BIO-101"),
            ("sub-acc", "Accounting", "হিসাববিজ্ঞান", "ACC-101"),
            ("sub-bus", "Business Studies", "ব্যবসায় শিক্ষা", "BUS-101"),
            ("sub-eng", "English", "ইংরেজি", "ENG-101"),
            ("sub-ban", "Bangla", "বাংলা", "BAN-101"),
            ("sub-sci", "General Science", "সাধারণ বিজ্ঞান", "SCI-101"),
        ];

        let mut ids = Vec::new();
        for (id, name, name_bn, code) in subjects {
            conn.execute(
                "INSERT INTO subjects (id, name, name_bn, code, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                params![id, name, name_bn, code, now_utc],
            )?;
            ids.push(id.to_string());
        }
        Ok(ids)
    }

    fn seed_programs_and_batches(
        &mut self,
        conn: &Connection,
        staff_ids: &[String],
        _subject_ids: &[String],
        now_utc: &str,
    ) -> Result<(Vec<String>, Vec<BatchInfo>), DbError> {
        let programs = [
            (
                "prog-hsc-sci",
                "HSC Science (Class 11-12)",
                "এইচএসসি বিজ্ঞান",
                "HSC-SCI",
                "11-12",
                "Science",
            ),
            (
                "prog-hsc-com",
                "HSC Commerce (Class 11-12)",
                "এইচএসসি ব্যবসায় শিক্ষা",
                "HSC-COM",
                "11-12",
                "Commerce",
            ),
            (
                "prog-ssc-gen",
                "SSC General (Class 9-10)",
                "এসএসসি সাধারণ",
                "SSC-GEN",
                "9-10",
                "General",
            ),
            (
                "prog-jsc-8",
                "Class 8 Junior (JSC Prep)",
                "৮ম শ্রেণি জুনিয়র",
                "JSC-8",
                "8",
                "General",
            ),
        ];

        let mut prog_ids = Vec::new();
        let mut batch_infos = Vec::new();

        for (p_id, p_name, p_name_bn, p_code, class_level, group_major) in programs {
            conn.execute(
                "INSERT INTO programs (
                    id, session_id, name, name_bn, code, class_level, group_major, status, created_at, updated_at
                ) VALUES (?1, 'sess-2026', ?2, ?3, ?4, ?5, ?6, 'active', ?7, ?7)",
                params![p_id, p_name, p_name_bn, p_code, class_level, group_major, now_utc],
            )?;
            prog_ids.push(p_id.to_string());

            // Create batches per program (AC-3: 12 batches total in demo, more in load)
            let batch_count = match self.profile {
                DemoProfile::Demo => 3,
                DemoProfile::Load => 50,
            };

            for b_idx in 1..=batch_count {
                let batch_id = format!("{}-b{:02}", p_id, b_idx);
                let shift = match b_idx % 3 {
                    0 => "morning",
                    1 => "day",
                    _ => "evening",
                };
                let room = format!("Room {:03}", 100 + (b_idx % 10));
                let teacher = &staff_ids[b_idx % staff_ids.len()];
                let batch_name =
                    format!("{} Batch {:02} ({})", p_code, b_idx, shift.to_uppercase());
                let batch_code = format!("{}-{:02}", p_code, b_idx);

                let capacity: u32 = 35;
                conn.execute(
                    "INSERT INTO batches (
                        id, program_id, name, code, teacher_staff_id, room_number, shift, capacity, start_date, status, created_at, updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, '2026-01-01', 'active', ?9, ?9)",
                    params![batch_id, p_id, batch_name, batch_code, teacher, room, shift, capacity, now_utc],
                )?;

                batch_infos.push(BatchInfo {
                    id: batch_id,
                    program_id: p_id.to_string(),
                    class_level: class_level.to_string(),
                    capacity,
                });
            }
        }

        Ok((prog_ids, batch_infos))
    }

    fn generate_students_and_guardians(
        &mut self,
        conn: &Connection,
        active_count: u32,
        archived_count: u32,
        now_utc: &str,
    ) -> Result<(Vec<String>, u32), DbError> {
        let total_count = active_count + archived_count;
        let mut student_ids = Vec::with_capacity(total_count as usize);

        // We prepare statements for chunked bulk inserts (AC-7)
        let mut guardian_stmt = conn.prepare(
            "INSERT INTO guardians (
                id, name, name_bn, relationship, phone_norm, present_address, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        )?;

        let mut student_stmt = conn.prepare(
            "INSERT INTO students (
                id, student_code, name, name_bn, gender, dob, phone_norm, emergency_phone_norm,
                present_address, status, admission_date, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
        )?;

        let mut link_stmt = conn.prepare(
            "INSERT INTO student_guardians (
                student_id, guardian_id, is_primary, receives_sms
            ) VALUES (?1, ?2, 1, 1)",
        )?;

        let mut guardian_count = 0u32;
        let mut current_guardian_id = String::new();
        let mut current_surname = ("", "");
        let mut current_address = "";
        let mut current_guardian_phone = String::new();
        let mut is_sibling_next = false;
        let mut previous_student_birth_year = 2008;

        // Transaction chunking of 1,000 records (AC-7)
        conn.execute("BEGIN TRANSACTION;", [])?;

        for i in 0..total_count {
            if i > 0 && i.is_multiple_of(1000) {
                conn.execute("COMMIT;", [])?;
                conn.execute("BEGIN TRANSACTION;", [])?;
            }

            let is_archived = i >= active_count;
            let student_id = format!("std-{:06}", i + 1);
            let student_code = format!("S{:06}", i + 1);

            // Gender distribution: ~52% male, 48% female (AC-2)
            let is_male = self.rng.gen_ratio(52, 100);
            let gender_str = if is_male { "male" } else { "female" };

            let (first_en, first_bn) = if is_male {
                MALE_FIRST_NAMES[self.rng.gen_range(0..MALE_FIRST_NAMES.len())]
            } else {
                FEMALE_FIRST_NAMES[self.rng.gen_range(0..FEMALE_FIRST_NAMES.len())]
            };

            let (surname_en, surname_bn, guardian_id, address, emergency_phone, birth_year) =
                if is_sibling_next && !current_guardian_id.is_empty() {
                    // This student is a sibling (AC-2): shares surname, address, guardian
                    is_sibling_next = false;
                    let age_gap = self.rng.gen_range(1..=4);
                    let birth_year = previous_student_birth_year + age_gap;

                    (
                        current_surname.0,
                        current_surname.1,
                        current_guardian_id.clone(),
                        current_address,
                        current_guardian_phone.clone(),
                        birth_year,
                    )
                } else {
                    // Create new guardian & family
                    let surname = FAMILY_SURNAMES[self.rng.gen_range(0..FAMILY_SURNAMES.len())];
                    current_surname = surname;
                    current_address = DHAKA_LOCATIONS[self.rng.gen_range(0..DHAKA_LOCATIONS.len())];

                    guardian_count += 1;
                    let g_id = format!("grd-{:06}", guardian_count);
                    current_guardian_id = g_id.clone();

                    let g_male_first =
                        MALE_FIRST_NAMES[self.rng.gen_range(0..MALE_FIRST_NAMES.len())];
                    let g_name = format!("{} {}", g_male_first.0, surname.0);
                    let g_name_bn = format!("{} {}", g_male_first.1, surname.1);

                    let raw_prefix = MOBILE_PREFIXES[self.rng.gen_range(0..MOBILE_PREFIXES.len())];
                    let raw_suffix: u32 = self.rng.gen_range(10000000..99999999);
                    let g_phone_raw = format!("{}{:08}", raw_prefix, raw_suffix % 100000000);
                    let g_phone_norm = phone::normalize_phone(&g_phone_raw)
                        .unwrap_or_else(|_| format!("88017{:08}", guardian_count));
                    current_guardian_phone = g_phone_norm.clone();

                    guardian_stmt.execute(params![
                        g_id,
                        g_name,
                        g_name_bn,
                        "father",
                        g_phone_norm,
                        current_address,
                        now_utc
                    ])?;

                    // AC-2: ~20% of students will be siblings with next student
                    if self.rng.gen_ratio(20, 100) && i + 1 < total_count {
                        is_sibling_next = true;
                    }

                    let base_birth_year = self.rng.gen_range(2006..=2010);
                    previous_student_birth_year = base_birth_year;

                    (
                        surname.0,
                        surname.1,
                        g_id,
                        current_address,
                        current_guardian_phone.clone(),
                        base_birth_year,
                    )
                };

            let student_name = format!("{} {}", first_en, surname_en);
            let student_name_bn = format!("{} {}", first_bn, surname_bn);
            let dob = format!(
                "{}-{:02}-{:02}",
                birth_year,
                self.rng.gen_range(1..=12),
                self.rng.gen_range(1..=28)
            );

            let s_prefix = MOBILE_PREFIXES[self.rng.gen_range(0..MOBILE_PREFIXES.len())];
            let s_phone_raw = format!("{}{:08}", s_prefix, self.rng.gen_range(10000000..99999999));
            let s_phone_norm = phone::normalize_phone(&s_phone_raw)
                .unwrap_or_else(|_| format!("88019{:08}", i + 1));

            let status = if is_archived { "archived" } else { "active" };
            let adm_year = if is_archived { 2023 } else { 2025 };
            let adm_date = format!("{}-01-15", adm_year);

            student_stmt.execute(params![
                student_id,
                student_code,
                student_name,
                student_name_bn,
                gender_str,
                dob,
                s_phone_norm,
                emergency_phone,
                address,
                status,
                adm_date,
                now_utc
            ])?;

            link_stmt.execute(params![student_id, guardian_id])?;
            student_ids.push(student_id);
        }

        conn.execute("COMMIT;", [])?;
        Ok((student_ids, guardian_count))
    }

    fn enroll_students(
        &mut self,
        conn: &Connection,
        active_student_ids: &[String],
        batches: &[BatchInfo],
        now_utc: &str,
    ) -> Result<Vec<EnrollmentInfo>, DbError> {
        let mut enrollments = Vec::new();
        let mut batch_roll_tracker: Vec<u32> = vec![0; batches.len()];

        let mut enroll_stmt = conn.prepare(
            "INSERT INTO enrollments (
                id, student_id, batch_id, roll, start_date, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, '2026-01-01', 'active', ?5, ?5)",
        )?;

        conn.execute("BEGIN TRANSACTION;", [])?;

        for (idx, student_id) in active_student_ids.iter().enumerate() {
            if idx > 0 && idx.is_multiple_of(1000) {
                conn.execute("COMMIT;", [])?;
                conn.execute("BEGIN TRANSACTION;", [])?;
            }

            // Assign to 1 or 2 batches respecting capacity bounds (AC-3: 15-40 students)
            let mut assigned_batch_idx = None;
            for offset in 0..batches.len() {
                let candidate = (idx + offset) % batches.len();
                if batch_roll_tracker[candidate] < batches[candidate].capacity {
                    assigned_batch_idx = Some(candidate);
                    break;
                }
            }

            let Some(primary_batch_idx) = assigned_batch_idx else {
                continue;
            };

            batch_roll_tracker[primary_batch_idx] += 1;
            let roll1 = batch_roll_tracker[primary_batch_idx];

            let e_id1 = format!("enr-{:06}-1", idx + 1);
            let b1 = &batches[primary_batch_idx];

            enroll_stmt.execute(params![e_id1, student_id, b1.id, roll1, now_utc])?;

            enrollments.push(EnrollmentInfo {
                id: e_id1,
                student_id: student_id.clone(),
                batch_id: b1.id.clone(),
                program_id: b1.program_id.clone(),
            });

            // ~30% enrolled in a secondary batch
            if self.rng.gen_ratio(30, 100) {
                let mut secondary_b_idx = None;
                for offset in 1..batches.len() {
                    let candidate = (primary_batch_idx + offset) % batches.len();
                    if batch_roll_tracker[candidate] < batches[candidate].capacity {
                        secondary_b_idx = Some(candidate);
                        break;
                    }
                }

                if let Some(sec_idx) = secondary_b_idx {
                    batch_roll_tracker[sec_idx] += 1;
                    let roll2 = batch_roll_tracker[sec_idx];

                    let e_id2 = format!("enr-{:06}-2", idx + 1);
                    let b2 = &batches[sec_idx];

                    enroll_stmt.execute(params![e_id2, student_id, b2.id, roll2, now_utc])?;

                    enrollments.push(EnrollmentInfo {
                        id: e_id2,
                        student_id: student_id.clone(),
                        batch_id: b2.id.clone(),
                        program_id: b2.program_id.clone(),
                    });
                }
            }
        }

        conn.execute("COMMIT;", [])?;
        Ok(enrollments)
    }

    fn simulate_financial_ledger(
        &mut self,
        conn: &Connection,
        enrollments: &[EnrollmentInfo],
        tuition_fee_head_id: &str,
        now_utc: &str,
    ) -> Result<(u32, u32), DbError> {
        let months_count = match self.profile {
            DemoProfile::Demo => 12,
            DemoProfile::Load => 36,
        };

        let mut charge_stmt = conn.prepare(
            "INSERT INTO charges (
                id, student_id, enrollment_id, fee_head_id, period, gross_amount_poisha,
                concession_amount_poisha, net_amount_poisha, due_date, source, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'generated', 'open', ?10, ?10)",
        )?;

        let mut pay_stmt = conn.prepare(
            "INSERT INTO payments (
                id, receipt_no, student_id, amount_poisha, method, business_date,
                received_at, idempotency_key, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'posted', ?9, ?9)",
        )?;

        let mut alloc_stmt = conn.prepare(
            "INSERT INTO payment_allocations (
                id, payment_id, charge_id, amount_poisha, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        let mut charge_count = 0u32;
        let mut payment_count = 0u32;

        conn.execute("BEGIN TRANSACTION;", [])?;

        for month_idx in 0..months_count {
            let year = 2025 + (month_idx / 12);
            let month = (month_idx % 12) + 1;
            let period = format!("{:04}-{:02}", year, month);
            let due_date = format!("{}-10", period);

            for enr in enrollments {
                charge_count += 1;
                if charge_count.is_multiple_of(1000) {
                    conn.execute("COMMIT;", [])?;
                    conn.execute("BEGIN TRANSACTION;", [])?;
                }

                let charge_id = format!("chg-{:07}", charge_count);
                let gross_amount: i64 = match enr.program_id.as_str() {
                    "prog-hsc-sci" => 350000,
                    "prog-hsc-com" => 300000,
                    "prog-ssc-gen" => 250000,
                    _ => 200000,
                };

                // AC-4: Simulates realistic collection statuses:
                // ~85% fully paid, 10% partially paid or overdue, 5% granted concessions
                let concession_poisha = if self.rng.gen_ratio(5, 100) {
                    50000 // 500 BDT concession
                } else {
                    0
                };
                let net_amount = gross_amount - concession_poisha;

                charge_stmt.execute(params![
                    charge_id,
                    enr.student_id,
                    enr.id,
                    tuition_fee_head_id,
                    period,
                    gross_amount,
                    concession_poisha,
                    net_amount,
                    due_date,
                    now_utc
                ])?;

                // Payment generation
                let is_paid = self.rng.gen_ratio(85, 100);
                let is_partial = !is_paid && self.rng.gen_ratio(50, 100);

                if is_paid || is_partial {
                    payment_count += 1;
                    let payment_id = format!("pay-{:07}", payment_count);
                    let receipt_no = format!("RCT-{:07}", payment_count);

                    let pay_amount = if is_paid { net_amount } else { net_amount / 2 };

                    // AC-4: Payments cluster between 5th and 10th of the month
                    let pay_day = if self.rng.gen_ratio(75, 100) {
                        self.rng.gen_range(5..=10)
                    } else if self.rng.gen_ratio(60, 100) {
                        self.rng.gen_range(11..=20)
                    } else {
                        self.rng.gen_range(21..=28)
                    };

                    let business_date = format!("{}-{:02}", period, pay_day);
                    let received_at = format!("{}T10:00:00Z", business_date);
                    let method = match self.rng.gen_range(0..4) {
                        0 => "cash",
                        1 => "bkash",
                        2 => "nagad",
                        _ => "bank",
                    };
                    let idempotency = format!("idem-pay-{}", payment_id);

                    pay_stmt.execute(params![
                        payment_id,
                        receipt_no,
                        enr.student_id,
                        pay_amount,
                        method,
                        business_date,
                        received_at,
                        idempotency,
                        now_utc
                    ])?;

                    let alloc_id = format!("alc-{:07}", payment_count);
                    alloc_stmt.execute(params![
                        alloc_id, payment_id, charge_id, pay_amount, now_utc
                    ])?;
                }
            }
        }

        // Maintain student_balances cache table
        conn.execute(
            "INSERT OR REPLACE INTO student_balances (
                student_id, total_charged_poisha, total_paid_poisha, total_adjusted_poisha,
                total_refunded_poisha, balance_poisha, updated_at
            )
            SELECT
                s.id,
                coalesce((SELECT sum(c.net_amount_poisha) FROM charges c WHERE c.student_id = s.id AND c.status = 'open'), 0) as charged,
                coalesce((SELECT sum(p.amount_poisha) FROM payments p WHERE p.student_id = s.id AND p.status = 'posted'), 0) as paid,
                0,
                0,
                coalesce((SELECT sum(c.net_amount_poisha) FROM charges c WHERE c.student_id = s.id AND c.status = 'open'), 0) -
                coalesce((SELECT sum(p.amount_poisha) FROM payments p WHERE p.student_id = s.id AND p.status = 'posted'), 0) as balance,
                ?1
            FROM students s",
            params![now_utc],
        )?;

        conn.execute("COMMIT;", [])?;
        Ok((charge_count, payment_count))
    }
}

#[derive(Debug, Clone)]
struct BatchInfo {
    id: String,
    program_id: String,
    #[allow(dead_code)]
    class_level: String,
    capacity: u32,
}

#[derive(Debug, Clone)]
struct EnrollmentInfo {
    id: String,
    student_id: String,
    #[allow(dead_code)]
    batch_id: String,
    program_id: String,
}
