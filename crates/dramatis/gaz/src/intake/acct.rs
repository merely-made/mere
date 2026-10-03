// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bounded acct identity comparison, with no network or IDNA construction.

use std::net::Ipv6Addr;

use super::IntakeError;

/// Normalize an ASCII `acct:user@host` or bare `user@host` for identity intake.
///
/// Scheme and DNS host case fold; account-name case stays significant.
/// Unreserved ASCII escapes decode, while other escapes retain uppercase hex.
/// Encoded controls/spaces are refused. Leading escapes are accepted, as in
/// RFC 7565's reported erratum 7998; this is not a full PRECIS validator.
/// Hosts are ASCII DNS/A-labels or bracketed IPv6 literals.
/// DNS trailing dots remain significant; provider alias equivalence is not
/// inferred. IPv6 literals use their parsed canonical spelling.
/// Constructing internationalized names and interpreting provider-specific
/// aliases belongs to the caller. No port, query, fragment or lookup is accepted.
pub fn normalize_acct_handle(value: &str) -> Result<String, IntakeError> {
    let invalid = || IntakeError::InvalidClaim("acct handle");
    let value = value.trim();
    if !value.is_ascii() {
        return Err(invalid());
    }
    let body = if value
        .get(..5)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("acct:"))
    {
        &value[5..]
    } else {
        value
    };
    let (user, host) = body.split_once('@').ok_or_else(invalid)?;
    if user.is_empty() || host.is_empty() || host.contains('@') {
        return Err(invalid());
    }
    let user = user_bytes(user).ok_or_else(invalid)?;
    let host = if host.starts_with('[') && host.ends_with(']') {
        format!(
            "[{}]",
            host[1..host.len() - 1]
                .parse::<Ipv6Addr>()
                .map_err(|_| invalid())?
        )
    } else {
        let labels = host.strip_suffix('.').unwrap_or(host);
        if labels.len() > 253
            || !labels.split('.').all(|label| {
                let bytes = label.as_bytes();
                !bytes.is_empty()
                    && bytes.len() <= 63
                    && bytes[0].is_ascii_alphanumeric()
                    && bytes[bytes.len() - 1].is_ascii_alphanumeric()
                    && bytes
                        .iter()
                        .all(|b| b.is_ascii_alphanumeric() || *b == b'-')
            })
        {
            return Err(invalid());
        }
        host.to_ascii_lowercase()
    };
    Ok(format!("acct:{user}@{host}"))
}

fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

fn sub_delim(byte: u8) -> bool {
    matches!(
        byte,
        b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'='
    )
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn user_bytes(user: &str) -> Option<String> {
    let bytes = user.as_bytes();
    let mut output = String::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' {
            let first = *bytes.get(index + 1)?;
            let second = *bytes.get(index + 2)?;
            let decoded = hex(first)? * 16 + hex(second)?;
            if decoded <= b' ' || decoded == 127 {
                return None;
            }
            if unreserved(decoded) {
                output.push(char::from(decoded));
            } else {
                output.push('%');
                output.push(char::from(first.to_ascii_uppercase()));
                output.push(char::from(second.to_ascii_uppercase()));
            }
            index += 3;
        } else if unreserved(byte) || sub_delim(byte) {
            output.push(char::from(byte));
            index += 1;
        } else {
            return None;
        }
    }
    Some(output)
}
