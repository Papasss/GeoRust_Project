#[cfg(test)]
mod tests {
    
    use super::*;

    fn sample(timestamp: u64, latitude: f64, longitude: f64) -> Coordinates {
        Coordinates {
            latitude,
            longitude,
            timestamp,
        }
    }

    fn test_config() -> AnalyticsConfig {
        AnalyticsConfig {
            movement_threshold_meters: 1.0,
            pause_threshold: Duration::from_secs(180),
        }
    }

    #[test]
    fn empty_history_returns_zero_statistics() {
        let stats = analyze_movement(
            &[],
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert_eq!(stats.path.len(), 0);
        assert_eq!(stats.total_distance_km, 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 0);
        assert_eq!(stats.pause_duration.as_secs(), 0);
        assert_eq!(stats.average_speed_kmh, 0.0);
    }

    #[test]
    fn single_position_returns_zero_statistics() {
        let history = vec![sample(0, 45.0, 7.0)];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert_eq!(stats.path.len(), 1);
        assert_eq!(stats.total_distance_km, 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 0);
        assert_eq!(stats.pause_duration.as_secs(), 0);
        assert_eq!(stats.average_speed_kmh, 0.0);
    }

    #[test]
    fn movement_is_counted_correctly() {
        let history = vec![
            sample(0, 45.0000, 7.0000),
            sample(30, 45.0010, 7.0010),
        ];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert!(stats.total_distance_km > 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 30);
        assert_eq!(stats.pause_duration.as_secs(), 0);
        assert!(stats.average_speed_kmh > 0.0);
    }

    #[test]
    fn pause_of_three_minutes_is_counted() {
        let history = vec![
            sample(0, 45.0000, 7.0000),
            sample(30, 45.0000, 7.0000),
            sample(60, 45.0000, 7.0000),
            sample(90, 45.0000, 7.0000),
            sample(120, 45.0000, 7.0000),
            sample(150, 45.0000, 7.0000),
            sample(180, 45.0000, 7.0000),
        ];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert_eq!(stats.total_distance_km, 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 0);
        assert_eq!(stats.pause_duration.as_secs(), 180);
        assert_eq!(stats.average_speed_kmh, 0.0);
    }

    #[test]
    fn pause_shorter_than_three_minutes_is_not_counted() {
        let history = vec![
            sample(0, 45.0000, 7.0000),
            sample(60, 45.0000, 7.0000),
        ];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert_eq!(stats.total_distance_km, 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 0);
        assert_eq!(stats.pause_duration.as_secs(), 0);
    }

    #[test]
    fn samples_outside_interval_are_ignored() {
        let history = vec![
            sample(0, 45.0000, 7.0000),
            sample(30, 45.0010, 7.0010),
            sample(5000, 46.0000, 8.0000),
        ];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 100,
            },
            test_config(),
        );

        assert_eq!(stats.path.len(), 2);
        assert!(stats.total_distance_km > 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 30);
    }
}

