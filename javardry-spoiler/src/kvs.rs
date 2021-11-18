use std::collections::HashMap;

use anyhow::{ensure, Context};
use log::warn;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::util;

pub(crate) type Kvs = HashMap<String, String>;

pub(crate) fn parse(plaintext: impl AsRef<str>) -> anyhow::Result<Kvs> {
    // キーのみを正規表現で抽出する。
    // なお、キーと値を以下の正規表現一発で抽出するとかなり遅くなる模様:
    // \A([0-9a-zA-Z_]+)\s*=\s*"(.*)"\z
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\A[0-9a-zA-Z_]+").expect("regex should be valid"));

    let plaintext = plaintext.as_ref();

    let mut kvs = Kvs::new();

    for line in plaintext.lines() {
        let line = util::trim_ascii(line);
        if line.is_empty() {
            continue;
        }

        // 先頭のキー文字列を抽出。
        let m = RE
            .find_at(line, 0)
            .with_context(|| format!("invalid line: {}", line))?;
        let (key, line) = line.split_at(m.end());

        // 直後の空白を除去。
        let line = util::trim_start_ascii(line);

        // '=' を読み飛ばす。
        ensure!(line.starts_with('='), "invalid line: {}", line);
        let line = &line[1..];

        // 直後の空白を除去。
        let line = util::trim_start_ascii(line);

        // '"' を読み飛ばす。
        ensure!(line.starts_with('"'), "invalid line: {}", line);
        let line = &line[1..];

        // 末尾が '"' であることを確認し、その直前までを値として抽出。
        ensure!(line.ends_with('"'), "invalid line: {}", line);
        let value = &line[..line.len() - 1];

        // キーの重複がある場合、後に現れた方を優先する。
        if let Some(value_old) = kvs.insert(key.to_owned(), value.to_owned()) {
            warn!("ignored duplicate entry: ({}, {})", key, value_old);
        }
    }

    Ok(kvs)
}

pub(crate) trait KvsExt {
    /// 必須キー key に対応する値を得る。key が存在しなければエラーを返す。
    fn get_expect(&self, key: impl AsRef<str>) -> anyhow::Result<&str>;

    /// key が存在すれば対応する値を、存在しなければ default を返す。
    fn get_or(&self, key: impl AsRef<str>, default: &'static str) -> &str;

    /// 連番キー ("Item0", "Item1", ... など) に対応する値のイテレータを返す。
    fn iter_seq(&self, key_prefix: impl Into<String>) -> Box<dyn Iterator<Item = &str> + '_>;
}

impl KvsExt for Kvs {
    fn get_expect(&self, key: impl AsRef<str>) -> anyhow::Result<&str> {
        let key = key.as_ref();

        self.get(key)
            .map(String::as_str)
            .with_context(|| format!("mandatory key not found: {}", key))
    }

    fn get_or(&self, key: impl AsRef<str>, default: &'static str) -> &str {
        let key = key.as_ref();

        self.get(key).map_or(default, String::as_str)
    }

    fn iter_seq(&self, key_prefix: impl Into<String>) -> Box<dyn Iterator<Item = &str> + '_> {
        use std::fmt::Write as _;

        let mut key = key_prefix.into();
        let prefix_len = key.len();
        let mut i = 0;

        let it = std::iter::from_fn(move || {
            key.truncate(prefix_len);
            write!(key, "{}", i).expect("write to String should succeed");

            i += 1;

            self.get(&key).map(String::as_str)
        });

        Box::new(it)
    }
}
