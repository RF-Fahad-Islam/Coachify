//! Converts currency amounts in poisha (integer 1/100 Taka) to words
//! in both English and Bengali (Bangla).

const EN_UNITS: &[&str] = &[
    "",
    "One",
    "Two",
    "Three",
    "Four",
    "Five",
    "Six",
    "Seven",
    "Eight",
    "Nine",
    "Ten",
    "Eleven",
    "Twelve",
    "Thirteen",
    "Fourteen",
    "Fifteen",
    "Sixteen",
    "Seventeen",
    "Eighteen",
    "Nineteen",
];

const EN_TENS: &[&str] = &[
    "", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety",
];

const BN_NUMBERS: &[&str] = &[
    "শূন্য",
    "এক",
    "দুই",
    "তিন",
    "চার",
    "পাঁচ",
    "ছয়",
    "সাত",
    "আট",
    "নয়",
    "দশ",
    "এগারো",
    "বারো",
    "তেরো",
    "চৌদ্দ",
    "পনেরো",
    "ষোলো",
    "সতেরো",
    "আঠারো",
    "উনিশ",
    "বিশ",
    "একুশ",
    "বাইশ",
    "তেইশ",
    "চব্বিশ",
    "পঁচিশ",
    "ছাব্বিশ",
    "সাতাশ",
    "আঠাশ",
    "উনত্রিশ",
    "ত্রিশ",
    "একত্রিশ",
    "বত্রিশ",
    "তেত্রিশ",
    "চৌত্রিশ",
    "পঁয়ত্রিশ",
    "ছত্রিশ",
    "সাঁইত্রিশ",
    "আটত্রিশ",
    "উনচল্লিশ",
    "চল্লিশ",
    "একচল্লিশ",
    "বিয়াল্লিশ",
    "তেতাল্লিশ",
    "চুয়াল্লিশ",
    "পঁয়তাল্লিশ",
    "ছেচল্লিশ",
    "সাতচল্লিশ",
    "আটচল্লিশ",
    "উনপঞ্চাশ",
    "পঞ্চাশ",
    "একান্ন",
    "বায়ান্ন",
    "তিপ্পান্ন",
    "চুয়ান্ন",
    "পঞ্চান্ন",
    "ছাপ্পান্ন",
    "সাতান্ন",
    "আটান্ন",
    "উনষাট",
    "ষাট",
    "একষট্টি",
    "বাষট্টি",
    "তেষট্টি",
    "চৌষট্টি",
    "পঁয়ষট্টি",
    "ছেষট্টি",
    "সাতষট্টি",
    "আটষট্টি",
    "উনসত্তর",
    "সত্তর",
    "একাত্তর",
    "বাহাত্তর",
    "তিয়াত্তর",
    "চুয়াত্তর",
    "পঁচাত্তর",
    "ছিয়াত্তর",
    "সাতাত্তর",
    "আটাত্তর",
    "উনআশি",
    "আশি",
    "একাশি",
    "বিরাশি",
    "তিরাশি",
    "চুরাশি",
    "পঁচাশি",
    "ছিয়াশি",
    "সাতাশি",
    "আটাশি",
    "উননব্বই",
    "নব্বই",
    "একানব্বই",
    "বিরানব্বই",
    "তিরানব্বই",
    "চুরানব্বই",
    "পঁচানব্বই",
    "ছিয়ানব্বই",
    "সাতানব্বই",
    "আটানব্বই",
    "নিরানব্বই",
];

fn english_under_thousand(n: u64) -> String {
    let mut parts = Vec::new();
    let hundreds = n / 100;
    let rem = n % 100;

    if hundreds > 0 {
        parts.push(format!("{} Hundred", EN_UNITS[hundreds as usize]));
    }

    if rem > 0 {
        if rem < 20 {
            parts.push(EN_UNITS[rem as usize].to_string());
        } else {
            let tens = rem / 10;
            let units = rem % 10;
            if units > 0 {
                parts.push(format!(
                    "{} {}",
                    EN_TENS[tens as usize], EN_UNITS[units as usize]
                ));
            } else {
                parts.push(EN_TENS[tens as usize].to_string());
            }
        }
    }

    parts.join(" ")
}

fn english_integer(mut n: u64) -> String {
    if n == 0 {
        return "Zero".to_string();
    }

    let mut parts = Vec::new();

    // Indian / Bangladeshi grouping: Crore (10,000,000), Lakh (100,000), Thousand (1,000), Hundred
    if n >= 10_000_000 {
        let crore = n / 10_000_000;
        parts.push(format!("{} Crore", english_integer(crore)));
        n %= 10_000_000;
    }

    if n >= 100_000 {
        let lakh = n / 100_000;
        parts.push(format!("{} Lakh", english_under_thousand(lakh)));
        n %= 100_000;
    }

    if n >= 1_000 {
        let thousand = n / 1_000;
        parts.push(format!("{} Thousand", english_under_thousand(thousand)));
        n %= 1_000;
    }

    if n > 0 {
        parts.push(english_under_thousand(n));
    }

    parts.join(" ")
}

pub fn amount_to_words_en(amount_poisha: i64) -> String {
    if amount_poisha == 0 {
        return "Zero Taka Only".to_string();
    }

    let is_negative = amount_poisha < 0;
    let abs_amount = amount_poisha.unsigned_abs();
    let taka = abs_amount / 100;
    let poisha = abs_amount % 100;

    let mut result = String::new();
    if is_negative {
        result.push_str("Minus ");
    }

    if taka > 0 {
        result.push_str(&english_integer(taka));
        result.push_str(" Taka");
        if poisha > 0 {
            result.push_str(" and ");
            result.push_str(&english_under_thousand(poisha));
            result.push_str(" Poisha");
        }
    } else if poisha > 0 {
        result.push_str(&english_under_thousand(poisha));
        result.push_str(" Poisha");
    } else {
        result.push_str("Zero Taka");
    }

    result.push_str(" Only");
    result
}

fn bengali_integer(mut n: u64) -> String {
    if n == 0 {
        return "শূন্য".to_string();
    }

    let mut parts = Vec::new();

    if n >= 10_000_000 {
        let crore = n / 10_000_000;
        parts.push(format!("{} কোটি", bengali_integer(crore)));
        n %= 10_000_000;
    }

    if n >= 100_000 {
        let lakh = n / 100_000;
        if lakh <= 99 {
            parts.push(format!("{} লক্ষ", BN_NUMBERS[lakh as usize]));
        } else {
            parts.push(format!("{} লক্ষ", bengali_integer(lakh)));
        }
        n %= 100_000;
    }

    if n >= 1_000 {
        let thousand = n / 1_000;
        if thousand <= 99 {
            parts.push(format!("{} হাজার", BN_NUMBERS[thousand as usize]));
        } else {
            parts.push(format!("{} হাজার", bengali_integer(thousand)));
        }
        n %= 1_000;
    }

    if n >= 100 {
        let hundred = n / 100;
        parts.push(format!("{} শত", BN_NUMBERS[hundred as usize]));
        n %= 100;
    }

    if n > 0 && n <= 99 {
        parts.push(BN_NUMBERS[n as usize].to_string());
    }

    parts.join(" ")
}

pub fn amount_to_words_bn(amount_poisha: i64) -> String {
    if amount_poisha == 0 {
        return "শূন্য টাকা মাত্র".to_string();
    }

    let is_negative = amount_poisha < 0;
    let abs_amount = amount_poisha.unsigned_abs();
    let taka = abs_amount / 100;
    let poisha = abs_amount % 100;

    let mut result = String::new();
    if is_negative {
        result.push_str("মাইনাস ");
    }

    if taka > 0 {
        result.push_str(&bengali_integer(taka));
        result.push_str(" টাকা");
        if poisha > 0 {
            result.push(' ');
            if poisha <= 99 {
                result.push_str(BN_NUMBERS[poisha as usize]);
            } else {
                result.push_str(&bengali_integer(poisha));
            }
            result.push_str(" পয়সা");
        }
    } else if poisha > 0 {
        if poisha <= 99 {
            result.push_str(BN_NUMBERS[poisha as usize]);
        } else {
            result.push_str(&bengali_integer(poisha));
        }
        result.push_str(" পয়সা");
    } else {
        result.push_str("শূন্য টাকা");
    }

    result.push_str(" মাত্র");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_amount_words() {
        assert_eq!(amount_to_words_en(0), "Zero Taka Only");
        assert_eq!(
            amount_to_words_en(220000),
            "Two Thousand Two Hundred Taka Only"
        );
        assert_eq!(
            amount_to_words_en(150050),
            "One Thousand Five Hundred Taka and Fifty Poisha Only"
        );
        assert_eq!(amount_to_words_en(50), "Fifty Poisha Only");
        assert_eq!(amount_to_words_en(10000000), "One Lakh Taka Only");
        assert_eq!(amount_to_words_en(1000000000), "One Crore Taka Only");
    }

    #[test]
    fn test_bengali_amount_words() {
        assert_eq!(amount_to_words_bn(0), "শূন্য টাকা মাত্র");
        assert_eq!(amount_to_words_bn(220000), "দুই হাজার দুই শত টাকা মাত্র");
        assert_eq!(amount_to_words_bn(150050), "এক হাজার পাঁচ শত টাকা পঞ্চাশ পয়সা মাত্র");
        assert_eq!(amount_to_words_bn(50), "পঞ্চাশ পয়সা মাত্র");
        assert_eq!(amount_to_words_bn(10000000), "এক লক্ষ টাকা মাত্র");
    }
}
