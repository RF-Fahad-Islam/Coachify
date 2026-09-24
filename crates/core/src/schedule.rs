use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

pub const VALID_DAYS: &[&str] = &[
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    #[error("Invalid day of week: '{0}'. Allowed: Sunday through Saturday")]
    InvalidDay(String),
    #[error("Invalid time format: '{0}'. Expected 'HH:MM' (24-hour)")]
    InvalidTimeFormat(String),
    #[error("Start time '{start}' must be before end time '{end}'")]
    StartNotBeforeEnd { start: String, end: String },
    #[error("JSON serialization or deserialization error: {0}")]
    Json(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ScheduleSlot {
    pub day_of_week: String,
    pub start_time: String,
    pub end_time: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ScheduleConflictItem {
    pub category: String, // "teacher" | "room"
    pub message: String,
    pub conflicting_batch_id: String,
    pub conflicting_batch_name: String,
    pub day_of_week: String,
    pub existing_time_slot: String,
    pub candidate_time_slot: String,
    pub resource_name: String,
}

/// Normalizes day string (e.g. "sunday" -> "Sunday") and returns error if invalid.
pub fn normalize_day(day: &str) -> Result<String, ScheduleError> {
    let trimmed = day.trim();
    for valid in VALID_DAYS {
        if valid.eq_ignore_ascii_case(trimmed) {
            return Ok(valid.to_string());
        }
    }
    Err(ScheduleError::InvalidDay(day.to_string()))
}

/// Parses "HH:MM" into minutes from midnight (0..1439).
pub fn parse_time_minutes(time: &str) -> Result<u32, ScheduleError> {
    let parts: Vec<&str> = time.trim().split(':').collect();
    if parts.len() != 2 {
        return Err(ScheduleError::InvalidTimeFormat(time.to_string()));
    }
    let hour: u32 = parts[0]
        .parse()
        .map_err(|_| ScheduleError::InvalidTimeFormat(time.to_string()))?;
    let minute: u32 = parts[1]
        .parse()
        .map_err(|_| ScheduleError::InvalidTimeFormat(time.to_string()))?;

    if hour >= 24 || minute >= 60 {
        return Err(ScheduleError::InvalidTimeFormat(time.to_string()));
    }

    Ok(hour * 60 + minute)
}

/// Validates a single schedule slot.
pub fn validate_slot(slot: &ScheduleSlot) -> Result<(), ScheduleError> {
    normalize_day(&slot.day_of_week)?;
    let start_mins = parse_time_minutes(&slot.start_time)?;
    let end_mins = parse_time_minutes(&slot.end_time)?;

    if start_mins >= end_mins {
        return Err(ScheduleError::StartNotBeforeEnd {
            start: slot.start_time.clone(),
            end: slot.end_time.clone(),
        });
    }

    Ok(())
}

/// Validates an entire list of schedule slots and ensures they don't self-conflict.
pub fn validate_schedule(slots: &[ScheduleSlot]) -> Result<(), ScheduleError> {
    for (i, slot) in slots.iter().enumerate() {
        validate_slot(slot)?;

        let day_norm = normalize_day(&slot.day_of_week)?;
        let start_mins = parse_time_minutes(&slot.start_time)?;
        let end_mins = parse_time_minutes(&slot.end_time)?;

        // Check self-collision within candidate slots
        for other in slots.iter().skip(i + 1) {
            let other_day = normalize_day(&other.day_of_week)?;
            if day_norm == other_day {
                let other_start = parse_time_minutes(&other.start_time)?;
                let other_end = parse_time_minutes(&other.end_time)?;

                if start_mins < other_end && end_mins > other_start {
                    return Err(ScheduleError::Json(format!(
                        "Self-conflicting schedule slots on {}: {}-{} overlaps with {}-{}",
                        day_norm, slot.start_time, slot.end_time, other.start_time, other.end_time
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Returns true if two slots overlap in day and time interval.
pub fn slots_overlap(a: &ScheduleSlot, b: &ScheduleSlot) -> bool {
    let day_a = match normalize_day(&a.day_of_week) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let day_b = match normalize_day(&b.day_of_week) {
        Ok(d) => d,
        Err(_) => return false,
    };

    if day_a != day_b {
        return false;
    }

    let start_a = match parse_time_minutes(&a.start_time) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let end_a = match parse_time_minutes(&a.end_time) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let start_b = match parse_time_minutes(&b.start_time) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let end_b = match parse_time_minutes(&b.end_time) {
        Ok(m) => m,
        Err(_) => return false,
    };

    start_a < end_b && end_a > start_b
}

struct ExistingBatchSchedule {
    id: String,
    name: String,
    teacher_staff_id: Option<String>,
    teacher_name: Option<String>,
    room_number: Option<String>,
    slots: Vec<ScheduleSlot>,
}

/// Detects teacher double-booking (lead and subject-level) and room overlap across all active
/// batches in SQLite.
///
/// Checks three collision categories:
/// 1. **Lead teacher conflict** – `teacher_staff_id` on the batch row.
/// 2. **Subject teacher conflict** – `teacher_staff_id` in `batch_subjects` rows (AC-4).
/// 3. **Room conflict** – `room_number` on the batch row or per-slot `room` override.
pub fn detect_conflicts(
    conn: &Connection,
    candidate_batch_id: Option<&str>,
    candidate_teacher_id: Option<&str>,
    candidate_room: Option<&str>,
    candidate_slots: &[ScheduleSlot],
) -> Result<Vec<ScheduleConflictItem>, rusqlite::Error> {
    if candidate_slots.is_empty() {
        return Ok(vec![]);
    }

    let mut stmt = conn.prepare(
        r#"
        SELECT b.id, b.name, b.teacher_staff_id, b.room_number, b.schedule_json, s.name as teacher_name
        FROM batches b
        LEFT JOIN staff s ON s.id = b.teacher_staff_id
        WHERE b.status = 'active'
        "#,
    )?;

    let active_batches = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let teacher_staff_id: Option<String> = row.get(2)?;
            let room_number: Option<String> = row.get(3)?;
            let schedule_json: String = row.get(4)?;
            let teacher_name: Option<String> = row.get(5)?;

            let slots: Vec<ScheduleSlot> = serde_json::from_str(&schedule_json).unwrap_or_default();

            Ok(ExistingBatchSchedule {
                id,
                name,
                teacher_staff_id,
                teacher_name,
                room_number,
                slots,
            })
        })?
        .filter_map(Result::ok)
        .filter(|b| {
            if let Some(cand_id) = candidate_batch_id {
                b.id != cand_id
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    let mut conflicts = Vec::new();

    let clean_cand_teacher = candidate_teacher_id
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let clean_cand_room = candidate_room.map(|s| s.trim()).filter(|s| !s.is_empty());

    for candidate_slot in candidate_slots {
        let eff_cand_room = candidate_slot
            .room
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .or(clean_cand_room);

        for existing in &active_batches {
            let clean_exist_teacher = existing
                .teacher_staff_id
                .as_deref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());
            let clean_exist_room = existing
                .room_number
                .as_deref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());

            for exist_slot in &existing.slots {
                if !slots_overlap(candidate_slot, exist_slot) {
                    continue;
                }

                let eff_exist_room = exist_slot
                    .room
                    .as_deref()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .or(clean_exist_room);

                // 1. Lead Teacher Collision
                if let (Some(cand_t), Some(exist_t)) = (clean_cand_teacher, clean_exist_teacher) {
                    if cand_t == exist_t {
                        let teacher_display = existing
                            .teacher_name
                            .as_deref()
                            .unwrap_or("Assigned Teacher");

                        conflicts.push(ScheduleConflictItem {
                            category: "teacher".to_string(),
                            message: format!(
                                "Teacher '{}' is already scheduled for '{}' on {} ({}-{})",
                                teacher_display,
                                existing.name,
                                exist_slot.day_of_week,
                                exist_slot.start_time,
                                exist_slot.end_time
                            ),
                            conflicting_batch_id: existing.id.clone(),
                            conflicting_batch_name: existing.name.clone(),
                            day_of_week: candidate_slot.day_of_week.clone(),
                            existing_time_slot: format!(
                                "{}-{}",
                                exist_slot.start_time, exist_slot.end_time
                            ),
                            candidate_time_slot: format!(
                                "{}-{}",
                                candidate_slot.start_time, candidate_slot.end_time
                            ),
                            resource_name: teacher_display.to_string(),
                        });
                    }
                }

                // 2. Room Collision
                if let (Some(c_rm), Some(e_rm)) = (eff_cand_room, eff_exist_room) {
                    if c_rm.eq_ignore_ascii_case(e_rm) {
                        conflicts.push(ScheduleConflictItem {
                            category: "room".to_string(),
                            message: format!(
                                "Room '{}' is already booked for '{}' on {} ({}-{})",
                                e_rm,
                                existing.name,
                                exist_slot.day_of_week,
                                exist_slot.start_time,
                                exist_slot.end_time
                            ),
                            conflicting_batch_id: existing.id.clone(),
                            conflicting_batch_name: existing.name.clone(),
                            day_of_week: candidate_slot.day_of_week.clone(),
                            existing_time_slot: format!(
                                "{}-{}",
                                exist_slot.start_time, exist_slot.end_time
                            ),
                            candidate_time_slot: format!(
                                "{}-{}",
                                candidate_slot.start_time, candidate_slot.end_time
                            ),
                            resource_name: e_rm.to_string(),
                        });
                    }
                }
            }
        }
    }

    // 3. Subject-teacher collision (AC-4): the candidate teacher may be assigned as a
    //    per-subject teacher in another active batch via `batch_subjects`. Check all such
    //    batches for overlapping time slots with the candidate schedule.
    if let Some(cand_t) = clean_cand_teacher {
        let mut sub_stmt = conn.prepare(
            r#"
            SELECT DISTINCT b.id, b.name, b.schedule_json, s.name as teacher_name
            FROM batch_subjects bs
            JOIN batches b ON b.id = bs.batch_id
            JOIN staff s ON s.id = bs.teacher_staff_id
            WHERE bs.teacher_staff_id = ?
              AND b.status = 'active'
            "#,
        )?;

        let subject_batches = sub_stmt
            .query_map([cand_t], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let schedule_json: String = row.get(2)?;
                let teacher_name: Option<String> = row.get(3)?;
                let slots: Vec<ScheduleSlot> =
                    serde_json::from_str(&schedule_json).unwrap_or_default();
                Ok((id, name, teacher_name, slots))
            })?
            .filter_map(Result::ok)
            .filter(|(batch_id, _, _, _)| candidate_batch_id.is_none_or(|cid| batch_id != cid))
            .collect::<Vec<_>>();

        for (batch_id, batch_name, teacher_name, exist_slots) in &subject_batches {
            for candidate_slot in candidate_slots {
                for exist_slot in exist_slots {
                    if !slots_overlap(candidate_slot, exist_slot) {
                        continue;
                    }
                    let teacher_display = teacher_name.as_deref().unwrap_or("Subject Teacher");
                    conflicts.push(ScheduleConflictItem {
                        category: "teacher".to_string(),
                        message: format!(
                            "Teacher '{}' is already a subject teacher in '{}' on {} ({}-{})",
                            teacher_display,
                            batch_name,
                            exist_slot.day_of_week,
                            exist_slot.start_time,
                            exist_slot.end_time
                        ),
                        conflicting_batch_id: batch_id.clone(),
                        conflicting_batch_name: batch_name.clone(),
                        day_of_week: candidate_slot.day_of_week.clone(),
                        existing_time_slot: format!(
                            "{}-{}",
                            exist_slot.start_time, exist_slot.end_time
                        ),
                        candidate_time_slot: format!(
                            "{}-{}",
                            candidate_slot.start_time, candidate_slot.end_time
                        ),
                        resource_name: teacher_display.to_string(),
                    });
                }
            }
        }
    }

    Ok(conflicts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_parsing_and_normalization() {
        assert_eq!(normalize_day("sunday").unwrap(), "Sunday");
        assert_eq!(normalize_day("Thursday").unwrap(), "Thursday");
        assert!(normalize_day("Funday").is_err());

        assert_eq!(parse_time_minutes("08:00").unwrap(), 480);
        assert_eq!(parse_time_minutes("09:30").unwrap(), 570);
        assert!(parse_time_minutes("25:00").is_err());
        assert!(parse_time_minutes("08:61").is_err());
    }

    #[test]
    fn test_slots_overlap() {
        let slot1 = ScheduleSlot {
            day_of_week: "Sunday".to_string(),
            start_time: "08:00".to_string(),
            end_time: "09:30".to_string(),
            room: Some("Room 101".to_string()),
        };

        // Overlapping slot
        let slot2 = ScheduleSlot {
            day_of_week: "Sunday".to_string(),
            start_time: "09:00".to_string(),
            end_time: "10:30".to_string(),
            room: Some("Room 101".to_string()),
        };
        assert!(slots_overlap(&slot1, &slot2));

        // Non-overlapping adjacent slot
        let slot3 = ScheduleSlot {
            day_of_week: "Sunday".to_string(),
            start_time: "09:30".to_string(),
            end_time: "11:00".to_string(),
            room: Some("Room 101".to_string()),
        };
        assert!(!slots_overlap(&slot1, &slot3));

        // Different day slot
        let slot4 = ScheduleSlot {
            day_of_week: "Monday".to_string(),
            start_time: "08:00".to_string(),
            end_time: "09:30".to_string(),
            room: Some("Room 101".to_string()),
        };
        assert!(!slots_overlap(&slot1, &slot4));
    }
}
