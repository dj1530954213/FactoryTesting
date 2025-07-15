use chrono::{DateTime, FixedOffset, Local, Utc, TimeZone};

/// 东八区偏移秒数
pub const BJ_OFFSET_SECONDS: i32 = 8 * 3600;

/// 返回东八区 `FixedOffset` 对象
#[inline]
pub fn bj_offset() -> FixedOffset {
    FixedOffset::east_opt(BJ_OFFSET_SECONDS).expect("Valid offset")
}

/// 当前北京时间 `DateTime<FixedOffset>`
#[inline]
pub fn now_bj() -> DateTime<FixedOffset> {
    Local::now().with_timezone(&bj_offset())
}

/// 将 `DateTime<Utc>` 转换为北京时间
#[inline]
pub fn to_bj(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    dt.with_timezone(&bj_offset())
}

/// 将任意时区 DateTime 格式化为北京时间字符串
#[inline]
pub fn format_bj<Tz: TimeZone>(dt: DateTime<Tz>, fmt: &str) -> String {
    dt.with_timezone(&bj_offset()).format(fmt).to_string()
}
