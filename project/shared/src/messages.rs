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
        start_timestamp: i64,
        end_timestamp: i64,
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

    SendDirectMessage {to: String, text: String},
    IncomingDirectMessage {from: String, text: String},
    SendBroadcastMessage {text: String},
    IncomingBroadcastMessage {from: String, text: String},

    // Analytics
    AnalyticsRequest {field: AnalyticsField, period: AnalyticsPeriodMessage},
    AnalyticsResponse(String),
    AnalyticsErr(String),

    Text(String),
}