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
            // Keep last 60 data points
            if self.sparkline_data.len() > 60 {
                self.sparkline_data.remove(0);
            }
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

    /// Format an ISO 8601 reset time string into a human-readable countdown.
    pub fn format_reset_time(reset_at: Option<&str>) -> String {
        let Some(s) = reset_at else {
            return "—".to_string();
        };
        let Ok(dt) = s.parse::<DateTime<Utc>>() else {
            return s.to_string();
        };
        let now = Utc::now();
        if dt <= now {
            return "resetting…".to_string();
        }
        let secs = (dt - now).num_seconds();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;

        let local: DateTime<Local> = dt.into();
        let time_str = local.format("%H:%M").to_string();

        if hours > 0 {
            format!("resets in {}h {}m (at {})", hours, mins, time_str)
        } else {
            format!("resets in {}m (at {})", mins, time_str)
        }
    }
}
