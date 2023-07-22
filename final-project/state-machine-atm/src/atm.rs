use crate::traits::StateMachine;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// The keys on the ATM keypad
pub enum Key {
    One,
    Two,
    Three,
    Four,
    Enter,
}

/// Something you can do to the ATM
pub enum Action {
    SwipeCard(u64),
    PressKey(Key),
}

/// The various states of authentication possible with the ATM
#[derive(Debug, Clone, PartialEq)]
enum Auth {
    Waiting,
    Authenticating(u64),
    Authenticated,
}

impl Default for Auth {
    fn default() -> Self {
        Auth::Waiting
    }
}

impl From<Key> for &str {
    fn from(key: Key) -> Self {
        match key {
            Key::One => "1",
            Key::Two => "2",
            Key::Three => "3",
            Key::Four => "4",
            Key::Enter => "Enter",
        }
    }
}

impl StateMachine for Atm {
    type State = Auth;
    type Transition = Action;

    fn next_state(starting_state: &Self::State, t: &Self::Transition) -> Self::State {
        match (starting_state, t) {
            (Auth::Waiting, Action::SwipeCard(pin_hash)) => Auth::Authenticating(*pin_hash),
            (Auth::Authenticating(pin_hash), Action::PressKey(Key::Enter)) => {
                // Simulate PIN hash verification (using simple hashing for demonstration)
                let mut hasher = DefaultHasher::new();
                starting_state.hash(&mut hasher);
                let current_hash = hasher.finish();

                if current_hash == *pin_hash {
                    Auth::Authenticated
                } else {
                    Auth::Waiting
                }
            }
            (Auth::Authenticated, Action::PressKey(Key::Enter)) => {
                // Simulate cash withdrawal
                if let Some(amount) =
                    calculate_withdrawal_amount(&starting_state.keystroke_register)
                {
                    if starting_state.cash_inside >= amount {
                        // Update the cash inside the ATM after withdrawal
                        let new_cash_inside = starting_state.cash_inside - amount;
                        Auth::Waiting // Reset state back to waiting for the next transaction
                            .with_cash_inside(new_cash_inside)
                    } else {
                        Auth::Waiting
                    }
                } else {
                    Auth::Waiting
                }
            }
            _ => starting_state.clone(), // No state change, return the current state
        }
    }
}

fn calculate_withdrawal_amount(keystroke_register: &[Key]) -> Option<u64> {
    let mut amount_str = String::new();
    for key in keystroke_register {
        if let Key::Enter = key {
            break;
        }
        amount_str.push_str(&String::from(*key));
    }
    amount_str.parse().ok()
}

impl Atm {
    // Helper method to create a new `Atm` with updated `cash_inside` value
    fn with_cash_inside(&self, cash_inside: u64) -> Atm {
        Atm {
            cash_inside,
            expected_pin_hash: self.expected_pin_hash.clone(),
            keystroke_register: Vec::new(),
        }
    }
}

pub struct Atm {
    cash_inside: u64,
    expected_pin_hash: Auth,
    keystroke_register: Vec<Key>,
}

#[test]
fn sm_3_simple_swipe_card() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::SwipeCard(1234));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_swipe_card_again_part_way_through() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::SwipeCard(1234));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One, Key::Three],
    };
    let end = Atm::next_state(&start, &Action::SwipeCard(1234));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One, Key::Three],
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_press_key_before_card_swipe() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::One));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_enter_single_digit_of_pin() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::One));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One],
    };

    assert_eq!(end, expected);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One],
    };
    let end1 = Atm::next_state(&start, &Action::PressKey(Key::Two));
    let expected1 = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One, Key::Two],
    };

    assert_eq!(end1, expected1);
}

#[test]
fn sm_3_enter_wrong_pin() {
    // Create hash of pin
    let pin = vec![Key::One, Key::Two, Key::Three, Key::Four];
    let pin_hash = crate::hash(&pin);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(pin_hash),
        keystroke_register: vec![Key::Three, Key::Three, Key::Three, Key::Three],
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_enter_correct_pin() {
    // Create hash of pin
    let pin = vec![Key::One, Key::Two, Key::Three, Key::Four];
    let pin_hash = crate::hash(&pin);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(pin_hash),
        keystroke_register: vec![Key::One, Key::Two, Key::Three, Key::Four],
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_enter_single_digit_of_withdraw_amount() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::One));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: vec![Key::One],
    };

    assert_eq!(end, expected);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: vec![Key::One],
    };
    let end1 = Atm::next_state(&start, &Action::PressKey(Key::Four));
    let expected1 = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: vec![Key::One, Key::Four],
    };

    assert_eq!(end1, expected1);
}

#[test]
fn sm_3_try_to_withdraw_too_much() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: vec![Key::One, Key::Four],
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_withdraw_acceptable_amount() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: vec![Key::One],
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
    let expected = Atm {
        cash_inside: 9,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}
