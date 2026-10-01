// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn last_four_is_exactly_four_ascii_digits() {
    assert_eq!(LastFour::parse("0042").unwrap().as_str(), "0042");
    for bad in ["123", "12345", "12a4", "", " 123", "１２３４"] {
        assert_eq!(LastFour::parse(bad), Err(ValueError::LastFour), "{bad:?}");
    }
}

#[test]
fn country_codes_are_two_uppercase_letters() {
    assert_eq!(CountryCode::parse("NZ").unwrap().as_str(), "NZ");
    for bad in ["nz", "N", "NZL", "N1", "", "Ñ"] {
        assert_eq!(
            CountryCode::parse(bad),
            Err(ValueError::CountryCode),
            "{bad:?}"
        );
    }
}

#[test]
fn subdivision_codes_follow_iso_3166_2() {
    for good in ["US-CA", "GB-ENG", "FR-75", "NZ-AUK", "CN-11"] {
        let code = SubdivisionCode::parse(good).unwrap();
        assert_eq!(code.as_str(), good);
        assert_eq!(code.country().as_str(), &good[..2]);
    }
    for bad in [
        "US", "US-", "US-CALI", "us-CA", "US-ca", "USA-CA", "US_CA", "-CA",
    ] {
        assert_eq!(
            SubdivisionCode::parse(bad),
            Err(ValueError::SubdivisionCode),
            "{bad:?}"
        );
    }
}

#[test]
fn dates_are_real_calendar_days() {
    let date = Date::parse("2028-02-29").unwrap();
    assert_eq!((date.year(), date.month(), date.day()), (2028, 2, 29));
    assert_eq!(date.to_string(), "2028-02-29");
    assert!(
        Date::parse("2000-02-29").is_ok(),
        "divisible by 400 is a leap year"
    );
    for bad in [
        "2027-02-29", // not a leap year
        "1900-02-29", // divisible by 100, not by 400
        "2027-04-31",
        "2027-13-01",
        "2027-00-10",
        "2027-01-00",
        "2027-1-01",
        "27-01-01",
        "2027/01/01",
        "2027-01-01T00:00:00Z",
        "",
    ] {
        assert_eq!(Date::parse(bad), Err(ValueError::Date), "{bad:?}");
    }
    assert_eq!(Date::new(2027, 6, 31), Err(ValueError::Date));
}

#[test]
fn dates_order_by_time() {
    let earlier = Date::parse("2027-12-31").unwrap();
    let later = Date::parse("2028-01-01").unwrap();
    assert!(earlier < later);
}

#[test]
fn year_months_are_written_yyyy_mm() {
    let expiry = YearMonth::parse("2029-07").unwrap();
    assert_eq!((expiry.year(), expiry.month()), (2029, 7));
    assert_eq!(expiry.to_string(), "2029-07");
    for bad in [
        "2029-7",
        "2029-13",
        "2029-00",
        "29-07",
        "2029-07-01",
        "2029/07",
        "",
    ] {
        assert_eq!(YearMonth::parse(bad), Err(ValueError::YearMonth), "{bad:?}");
    }
}

#[test]
fn ssh_fingerprints_are_the_sha256_form() {
    let text = "SHA256:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU";
    assert_eq!(SshFingerprint::parse(text).unwrap().as_str(), text);
    for bad in [
        "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU",
        "MD5:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU",
        "SHA256:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuF",
        "SHA256:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
        "SHA256:47DEQpj8HBSa-_TImW+5JCeuQeRkm5NMpJWZG3hSuFU",
    ] {
        assert_eq!(
            SshFingerprint::parse(bad),
            Err(ValueError::SshFingerprint),
            "{bad:?}"
        );
    }
}

#[test]
fn every_value_is_text_in_json_and_in_postcard() {
    fn round_trip<T>(value: T, text: &str)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + fmt::Debug,
    {
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, format!("\"{text}\""));
        assert_eq!(serde_json::from_str::<T>(&json).unwrap(), value);

        let bytes = postcard::to_allocvec(&value).unwrap();
        assert_eq!(postcard::to_allocvec(text).unwrap(), bytes, "postcard text");
        assert_eq!(postcard::from_bytes::<T>(&bytes).unwrap(), value);
    }
    round_trip(LastFour::parse("4242").unwrap(), "4242");
    round_trip(CountryCode::parse("DE").unwrap(), "DE");
    round_trip(SubdivisionCode::parse("DE-BY").unwrap(), "DE-BY");
    round_trip(Date::parse("2031-10-01").unwrap(), "2031-10-01");
    round_trip(YearMonth::parse("2031-10").unwrap(), "2031-10");
    let fingerprint = "SHA256:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU";
    round_trip(SshFingerprint::parse(fingerprint).unwrap(), fingerprint);
}

#[test]
fn a_value_that_fails_its_check_does_not_load() {
    assert!(serde_json::from_str::<LastFour>("\"123\"").is_err());
    assert!(serde_json::from_str::<Date>("\"2027-02-29\"").is_err());
    let bytes = postcard::to_allocvec("12345").unwrap();
    assert!(postcard::from_bytes::<LastFour>(&bytes).is_err());
    let bytes = postcard::to_allocvec("2029-13").unwrap();
    assert!(postcard::from_bytes::<YearMonth>(&bytes).is_err());
}
