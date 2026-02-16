use crate::config::CronJobConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// A scheduled cron job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub enabled: bool,
    pub kind: String,
    pub prompt: Option<String>,
    pub session_target: Option<String>,
    pub channel: Option<String>,
    pub to: Option<String>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub run_count: u64,
}

impl CronJob {
    pub fn from_config(config: &CronJobConfig) -> Self {
        let id = config.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let schedule = config.schedule.clone().unwrap_or_default();
        let next_run = parse_next_run(&schedule);

        Self {
            id,
            name: config.name.clone().unwrap_or_default(),
            schedule,
            enabled: config.enabled.unwrap_or(true),
            kind: config.kind.clone().unwrap_or_else(|| "agentTurn".to_string()),
            prompt: config.prompt.clone(),
            session_target: config.session_target.clone(),
            channel: config.channel.clone(),
            to: config.to.clone(),
            last_run: None,
            next_run,
            run_count: 0,
        }
    }

    /// Check if this job should run now.
    pub fn should_run(&self, now: &DateTime<Utc>) -> bool {
        if !self.enabled {
            return false;
        }
        match &self.next_run {
            Some(next) => now >= next,
            None => false,
        }
    }

    /// Advance to the next scheduled run.
    pub fn advance(&mut self) {
        self.last_run = Some(Utc::now());
        self.run_count += 1;
        self.next_run = parse_next_run(&self.schedule);
    }
}

/// Parse a cron schedule string and compute the next run time.
fn parse_next_run(schedule: &str) -> Option<DateTime<Utc>> {
    if schedule.is_empty() {
        return None;
    }

    // Support simple interval format: "30m", "1h", "24h"
    if let Some(duration) = parse_interval(schedule) {
        return Some(Utc::now() + duration);
    }

    // Try standard cron expression
    match schedule.parse::<cron::Schedule>() {
        Ok(sched) => {
            sched.upcoming(Utc).next()
        }
        Err(e) => {
            warn!("Invalid cron schedule '{}': {}", schedule, e);
            None
        }
    }
}

/// Parse interval strings like "30m", "1h", "24h", "60s".
fn parse_interval(s: &str) -> Option<chrono::Duration> {
    let s = s.trim();
    if s.ends_with('s') {
        s[..s.len()-1].parse::<i64>().ok().map(chrono::Duration::seconds)
    } else if s.ends_with('m') {
        s[..s.len()-1].parse::<i64>().ok().map(chrono::Duration::minutes)
    } else if s.ends_with('h') {
        s[..s.len()-1].parse::<i64>().ok().map(chrono::Duration::hours)
    } else if s.ends_with('d') {
        s[..s.len()-1].parse::<i64>().ok().map(chrono::Duration::days)
    } else {
        None
    }
}

/// The cron service manages scheduled jobs.
#[derive(Clone)]
pub struct CronService {
    jobs: Arc<RwLock<Vec<CronJob>>>,
    running: Arc<RwLock<bool>>,
}

impl CronService {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Load jobs from config.
    pub async fn load_from_config(&self, jobs: &[CronJobConfig]) {
        let mut store = self.jobs.write().await;
        store.clear();
        for job_config in jobs {
            store.push(CronJob::from_config(job_config));
        }
        info!("Loaded {} cron jobs", store.len());
    }

    /// Add a job.
    pub async fn add_job(&self, job: CronJob) {
        let mut jobs = self.jobs.write().await;
        jobs.push(job);
    }

    /// List all jobs.
    pub async fn list_jobs(&self) -> Vec<CronJob> {
        let jobs = self.jobs.read().await;
        jobs.clone()
    }

    /// Get a job by ID.
    pub async fn get_job(&self, id: &str) -> Option<CronJob> {
        let jobs = self.jobs.read().await;
        jobs.iter().find(|j| j.id == id).cloned()
    }

    /// Enable/disable a job.
    pub async fn set_enabled(&self, id: &str, enabled: bool) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
            job.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Remove a job.
    pub async fn remove_job(&self, id: &str) -> bool {
        let mut jobs = self.jobs.write().await;
        let len_before = jobs.len();
        jobs.retain(|j| j.id != id);
        jobs.len() < len_before
    }

    /// Check for due jobs and return them.
    pub async fn check_due_jobs(&self) -> Vec<CronJob> {
        let now = Utc::now();
        let mut jobs = self.jobs.write().await;
        let mut due = Vec::new();

        for job in jobs.iter_mut() {
            if job.should_run(&now) {
                due.push(job.clone());
                job.advance();
            }
        }

        due
    }

    /// Start the cron tick loop.
    pub async fn start(&self) {
        {
            let mut running = self.running.write().await;
            if *running {
                return;
            }
            *running = true;
        }

        let jobs = self.jobs.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            info!("Cron service started");
            loop {
                {
                    let is_running = running.read().await;
                    if !*is_running {
                        break;
                    }
                }

                // Check every 30 seconds
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                let now = Utc::now();
                let mut store = jobs.write().await;
                for job in store.iter_mut() {
                    if job.should_run(&now) {
                        info!("Cron job due: {} ({})", job.name, job.id);
                        job.advance();
                        // In a full implementation, this would trigger the agent run
                    }
                }
            }
            info!("Cron service stopped");
        });
    }

    /// Stop the cron service.
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_interval_seconds() {
        let d = parse_interval("60s").unwrap();
        assert_eq!(d.num_seconds(), 60);
    }

    #[test]
    fn parse_interval_minutes() {
        let d = parse_interval("30m").unwrap();
        assert_eq!(d.num_minutes(), 30);
    }

    #[test]
    fn parse_interval_hours() {
        let d = parse_interval("2h").unwrap();
        assert_eq!(d.num_hours(), 2);
    }

    #[test]
    fn parse_interval_days() {
        let d = parse_interval("1d").unwrap();
        assert_eq!(d.num_days(), 1);
    }

    #[test]
    fn parse_interval_invalid() {
        assert!(parse_interval("abc").is_none());
        assert!(parse_interval("").is_none());
    }

    #[test]
    fn cron_job_from_config() {
        let config = CronJobConfig {
            id: Some("test".into()),
            name: Some("Test Job".into()),
            schedule: Some("30m".into()),
            enabled: Some(true),
            kind: Some("agentTurn".into()),
            prompt: Some("Do something".into()),
            session_target: None,
            channel: None,
            to: None,
        };
        let job = CronJob::from_config(&config);
        assert_eq!(job.id, "test");
        assert_eq!(job.name, "Test Job");
        assert!(job.enabled);
        assert!(job.next_run.is_some());
    }

    #[test]
    fn cron_job_should_run() {
        let mut job = CronJob::from_config(&CronJobConfig {
            id: Some("t".into()),
            schedule: Some("1s".into()),
            enabled: Some(true),
            ..Default::default()
        });
        // Should not run yet (next_run is ~1s in future)
        let now = Utc::now();
        assert!(!job.should_run(&now));

        // Force next_run to past
        job.next_run = Some(now - chrono::Duration::seconds(1));
        assert!(job.should_run(&now));
    }

    #[test]
    fn cron_job_disabled() {
        let mut job = CronJob::from_config(&CronJobConfig {
            id: Some("t".into()),
            schedule: Some("1s".into()),
            enabled: Some(false),
            ..Default::default()
        });
        job.next_run = Some(Utc::now() - chrono::Duration::seconds(1));
        assert!(!job.should_run(&Utc::now()));
    }

    #[tokio::test]
    async fn cron_service_add_and_list() {
        let svc = CronService::new();
        svc.add_job(CronJob::from_config(&CronJobConfig {
            id: Some("j1".into()),
            name: Some("Job 1".into()),
            schedule: Some("1h".into()),
            ..Default::default()
        })).await;
        let jobs = svc.list_jobs().await;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, "j1");
    }

    #[tokio::test]
    async fn cron_service_remove() {
        let svc = CronService::new();
        svc.add_job(CronJob::from_config(&CronJobConfig {
            id: Some("j1".into()),
            ..Default::default()
        })).await;
        assert!(svc.remove_job("j1").await);
        assert!(!svc.remove_job("j1").await);
        assert_eq!(svc.list_jobs().await.len(), 0);
    }

    #[tokio::test]
    async fn cron_service_enable_disable() {
        let svc = CronService::new();
        svc.add_job(CronJob::from_config(&CronJobConfig {
            id: Some("j1".into()),
            enabled: Some(true),
            ..Default::default()
        })).await;
        assert!(svc.set_enabled("j1", false).await);
        let job = svc.get_job("j1").await.unwrap();
        assert!(!job.enabled);
    }

    #[tokio::test]
    async fn cron_service_load_from_config() {
        let svc = CronService::new();
        let configs = vec![
            CronJobConfig { id: Some("a".into()), schedule: Some("1h".into()), ..Default::default() },
            CronJobConfig { id: Some("b".into()), schedule: Some("2h".into()), ..Default::default() },
        ];
        svc.load_from_config(&configs).await;
        assert_eq!(svc.list_jobs().await.len(), 2);
    }
}
