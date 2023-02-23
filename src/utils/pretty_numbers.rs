/// make numbers more readable

const VALID_NUMERALS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
// TODO: pick a better name please.
pub trait PrettiableNumber {
    fn pretty(&self) -> String;
}

impl PrettiableNumber for String {
    fn pretty(&self) -> String {
        let mut pretty_number = Vec::<u8>::new();

        let iter = self.as_bytes().iter().rev().zip(1..);
        for (b, i) in iter {
            if VALID_NUMERALS.contains(b) {
                pretty_number.insert(0, *b);
                if i % 3 == 0 {
                    pretty_number.insert(0, b',');
                }
            } else {
                // throw an error , not a number.
            }
        }

        let Some(first) = pretty_number.first() else {
            panic!("couldnt fetch first byte to check if comma, number seems to be empty. why?");
        };

        if first.eq(&b',') {
            pretty_number.remove(0); // NOTE: bad shifts all elements to left
        }

        use std::str;

        let Ok(num) = str::from_utf8(&pretty_number) else {
            // at this point it should only be numbers and commas.
            // if its failed for some reason then i dunno why.
            // TODO: throw an error, for now i panic
            panic!("unknown error occured while trying to print")

        };

        num.to_owned().clone() // TODO: investiage how any given function(to_owned, clone) convert a type with referenced data to a type
    }
}

#[cfg(test)]
mod test {

    use super::PrettiableNumber;

    #[test]
    fn test_pretty_for_string() {
        assert_eq!("100,000", String::from("100000").pretty());
    }
}
