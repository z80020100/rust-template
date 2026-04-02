pub mod configs;
pub mod constant;
pub mod error;
pub mod logger;
pub mod threads;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use strum::EnumCount;

    #[tokio::test]
    async fn exit_codes_do_not_overlap_between_error_types() {
        use crate::error::ErrorCode;
        use crate::threads::ThreadCommand;
        use crate::threads::error::ThreadErrorCode;
        use tokio::sync::{broadcast, mpsc};
        use tracing::level_filters::LevelFilter;
        use tracing_subscriber::{Registry, reload};

        let join_err = {
            let handle = tokio::spawn(std::future::pending::<()>());
            handle.abort();
            handle.await.unwrap_err()
        };
        let reload_err = {
            let (layer, handle) = reload::Layer::<LevelFilter, Registry>::new(LevelFilter::INFO);
            drop(layer);
            handle.reload(LevelFilter::DEBUG).unwrap_err()
        };

        let shared: Vec<u8> = [
            ErrorCode::ConfigLoadFail(config::ConfigError::Message("test".into())),
            ErrorCode::LoggerLevelParseFail("invalid".parse::<LevelFilter>().unwrap_err()),
            ErrorCode::LoggerLevelReloadFail(reload_err),
        ]
        .iter()
        .map(|e| e.as_u8())
        .collect();

        let thread: Vec<u8> = [
            ThreadErrorCode::MpscUnboundChanI32SendFail(mpsc::error::SendError(0)),
            ThreadErrorCode::MpscUnboundChanRecvFail,
            ThreadErrorCode::MpmcChanThrCmdSendFail(broadcast::error::SendError(
                ThreadCommand::Stop,
            )),
            ThreadErrorCode::MpmcChanRecvFail(broadcast::error::RecvError::Closed),
            ThreadErrorCode::ThreadJoinFail(join_err),
            ThreadErrorCode::ShutdownTimeout,
        ]
        .iter()
        .map(|e| e.as_u8())
        .collect();

        assert_eq!(
            shared.len(),
            ErrorCode::COUNT,
            "Must cover all ErrorCode variants"
        );
        assert_eq!(
            thread.len(),
            ThreadErrorCode::COUNT,
            "Must cover all ThreadErrorCode variants"
        );

        let all: HashSet<u8> = shared.iter().chain(&thread).copied().collect();
        assert_eq!(
            all.len(),
            shared.len() + thread.len(),
            "Exit codes must not overlap: shared={shared:?} thread={thread:?}"
        );
    }
}
