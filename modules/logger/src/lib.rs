/// Инициализирует tracing subscriber для логирования
pub fn init() {
    tracing_subscriber::fmt::init();
}

/// Логирует ошибку с контекстом
pub fn log_error(context: &str, error: impl std::fmt::Display) {
    tracing::error!(context = %context, "{}", error);
}

/// Логирует предупреждение с контекстом
pub fn log_warning(context: &str, message: impl std::fmt::Display) {
    tracing::warn!(context = %context, "{}", message);
}

/// Логирует информационное сообщение с контекстом
pub fn log_info(context: &str, message: impl std::fmt::Display) {
    tracing::info!(context = %context, "{}", message);
}

