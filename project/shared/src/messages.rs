use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AnalyticsField {
    Path,
    TotalDistance,
    AverageSpeed,
    MovementDuration,
    PauseDuration,
    All,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AnalyticsPeriodMessage {
    CurrentDay,
    CurrentWeek,
    CurrentMonth,
    Custom {
        start_timestamp: u64,
        end_timestamp: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    Register { username: String, password: String },
    Login { username: String, password: String },

    RegisterOk,
    RegisterErr(String),
    LoginOk,
    LoginErr(String),

    // Analytics
    AnalyticsRequest {
        field: AnalyticsField,
        period: AnalyticsPeriodMessage,
    },
    AnalyticsResponse(String),
    AnalyticsErr(String),

    Text(String),
}