use crate::api::types::UsageData;
use chrono::{DateTime, Local, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Online,
    Offline,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct App {
    pub data: UsageData,
    pub sparkline_data: Vec<f64>,
    pub sparkline_resets: Vec<bool>,
    pending_reset: bool,
    pub connection: ConnectionStatus,
    pub refresh_interval: u64,
    pub plan_name: String,
    pub running: bool,
}

impl App {
    pub fn new(refresh_interval: u64, plan_name: String) -> Self {
        Self {
            data: UsageData::default(),
            sparkline_data: Vec::new(),
            sparkline_resets: Vec::new(),
            pending_reset: false,
            connection: ConnectionStatus::Disconnected,
            refresh_interval,
            plan_name,
            running: true,
        }
    }

    pub fn update_data(&mut self, data: UsageData) {
        // Push current session usage into sparkline history before updating
        if let Some(pct) = self.data.session_percent_used {
            self.sparkline_data.push(pct);
            self.sparkline_resets.push(self.pending_reset);
            self.pending_reset = false;
            // Keep last 60 data points
            if self.sparkline_data.len() > 60 {
                self.sparkline_data.remove(0);
                self.sparkline_resets.remove(0);
            }
        }
        // Detect reset: session_reset_at changed
        if self.data.session_reset_at.is_some()
            && data.session_reset_at.is_some()
            && self.data.session_reset_at != data.session_reset_at
        {
            self.pending_reset = true;
        }
        self.data = data;
        self.connection = ConnectionStatus::Online;
    }

    pub fn set_error(&mut self, is_network: bool) {
        self.connection = if is_network {
            ConnectionStatus::Offline
        } else {
            ConnectionStatus::Disconnected
        };
    }

    pub fn increase_interval(&mut self) {
        self.refresh_interval = (self.refresh_interval + 5).min(300);
    }

    pub fn decrease_interval(&mut self) {
        self.refresh_interval = self.refresh_interval.saturating_sub(5).max(5);
    }

    /// Format reset time like Claude-Usage-Tracker: "Resets Today 07:00" or "Resets Mar 19, 19:00"
    pub fn format_reset_time(reset_at: Option<&str>) -> String {
        let Some(s) = reset_at else {
            return "—".to_string();
        };
        let Ok(dt) = s.parse::<DateTime<Utc>>() else {
            return s.to_string();
        };
        let now_utc = Utc::now();
        if dt <= now_utc {
            return "Resetting…".to_string();
        }

        let local: DateTime<Local> = dt.into();
        let today: DateTime<Local> = Local::now();

        let time_str = local.format("%H:%M").to_string();

        if local.date_naive() == today.date_naive() {
            format!("Resets Today {}", time_str)
        } else if local.date_naive() == today.date_naive() + chrono::Duration::days(1) {
            format!("Resets Tomorrow {}", time_str)
        } else {
            format!("Resets {}", local.format("%b %d, %H:%M"))
        }
    }
}
