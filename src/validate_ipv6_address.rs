#![warn(clippy::pedantic)]

use super::character_classes::{
    DIGIT,
    HEXDIG,
};
use super::context::Context;
use super::error::Error;
use super::validate_ipv4_address::validate_ipv4_address;

enum MachineExitStatus<'a> {
    Error(Error),
    Ipv4Trailer(Shared<'a>),
}

impl<'a> From<Error> for MachineExitStatus<'a> {
    fn from(error: Error) -> Self {
        MachineExitStatus::Error(error)
    }
}

struct Shared<'a> {
    address: &'a str,
    num_groups: usize,
    num_digits: usize,
    double_colon_encountered: bool,
    potential_ipv4_address_start: usize,
}

enum State<'a> {
    NoGroupsYet(Shared<'a>),
    ColonButNoGroupsYet(Shared<'a>),
    AfterDoubleColon(Shared<'a>),
    InGroupNotIpv4(Shared<'a>),
    InGroupCouldBeIpv4(Shared<'a>),
    InGroupIpv4(Shared<'a>),
    ColonAfterGroup(Shared<'a>),
}

impl<'a> State<'a> {
    fn finalize(mut self) -> Result<(), Error> {
        match &mut self {
            Self::InGroupNotIpv4(state)
            | Self::InGroupCouldBeIpv4(state) => {
                // count trailing group
                state.num_groups += 1;
            },
            Self::InGroupIpv4(state) => {
                validate_ipv4_address(&state.address[state.potential_ipv4_address_start..])?;
                state.num_groups += 2;
            },
            _ => {},
        };
        match self {
            Self::ColonButNoGroupsYet(_)
            | Self::ColonAfterGroup(_) => Err(Error::TruncatedHost),

            Self::AfterDoubleColon(state)
            | Self::InGroupNotIpv4(state)
            | Self::InGroupCouldBeIpv4(state)
            | Self::InGroupIpv4(state)
            | Self::NoGroupsYet(state) => {
                match (state.double_colon_encountered, state.num_groups) {
                    (true, n) if n <= 7 => Ok(()),
                    (false, 8) => Ok(()),
                    (false, n) if n < 8 => Err(Error::TooFewAddressParts),
                    (_, _) => Err(Error::TooManyAddressParts),
                }
            }
        }
    }

    fn new(address: &'a str) -> Self {
        Self::NoGroupsYet(Shared{
            address,
            num_groups: 0,
            num_digits: 0,
            double_colon_encountered: false,
            potential_ipv4_address_start: 0,
        })
    }

    fn next(self, i: usize, c: char) -> Result<Self, MachineExitStatus<'a>> {
        match self {
            Self::NoGroupsYet(state) => Self::next_no_groups_yet(state, i, c),
            Self::ColonButNoGroupsYet(state) => Self::next_colon_but_no_groups_yet(state, c),
            Self::AfterDoubleColon(state) => Self::next_after_double_colon(state, i, c),
            Self::InGroupNotIpv4(state) => Self::next_in_group_not_ipv4(state, c),
            Self::InGroupCouldBeIpv4(state) => Self::next_in_group_could_be_ipv4(state, c),
            Self::InGroupIpv4(state) => Ok(Self::InGroupIpv4(state)),
            Self::ColonAfterGroup(state) => Self::next_colon_after_group(state, i, c),
        }
    }

    fn next_no_groups_yet(state: Shared<'a>, i: usize, c: char) -> Result<Self, MachineExitStatus> {
        let mut state = state;
        if c == ':' {
            Ok(Self::ColonButNoGroupsYet(state))
        } else if DIGIT.contains(&c) {
            state.potential_ipv4_address_start = i;
            state.num_digits = 1;
            Ok(Self::InGroupCouldBeIpv4(state))
        } else if HEXDIG.contains(&c) {
            state.num_digits = 1;
            Ok(Self::InGroupNotIpv4(state))
        } else {
            Err(Error::IllegalCharacter(Context::Ipv6Address).into())
        }
    }

    fn next_colon_but_no_groups_yet(state: Shared<'a>, c: char) -> Result<Self, MachineExitStatus> {
        let mut state = state;
        if c == ':' {
            state.double_colon_encountered = true;
            Ok(Self::AfterDoubleColon(state))
        } else {
            Err(Error::IllegalCharacter(Context::Ipv6Address).into())
        }
    }

    fn next_after_double_colon(state: Shared<'a>, i: usize, c: char) -> Result<Self, MachineExitStatus> {
        let mut state = state;
        state.num_digits += 1;
        if state.num_digits > 4 {
            Err(Error::TooManyDigits.into())
        } else if DIGIT.contains(&c) {
            state.potential_ipv4_address_start = i;
            Ok(Self::InGroupCouldBeIpv4(state))
        } else if HEXDIG.contains(&c) {
            Ok(Self::InGroupNotIpv4(state))
        } else {
            Err(Error::IllegalCharacter(Context::Ipv6Address).into())
        }
    }

    fn next_in_group_not_ipv4(state: Shared<'a>, c: char) -> Result<Self, MachineExitStatus> {
        let mut state = state;
        if c == ':' {
            state.num_digits = 0;
            state.num_groups += 1;
            Ok(Self::ColonAfterGroup(state))
        } else if HEXDIG.contains(&c) {
            state.num_digits += 1;
            if state.num_digits > 4 {
                Err(Error::TooManyDigits.into())
            } else {
                Ok(Self::InGroupNotIpv4(state))
            }
        } else {
            Err(Error::IllegalCharacter(Context::Ipv6Address).into())
        }
    }

    fn next_in_group_could_be_ipv4(state: Shared<'a>, c: char) -> Result<Self, MachineExitStatus> {
        let mut state = state;
        if c == ':' {
            state.num_digits = 0;
            state.num_groups += 1;
            Ok(Self::ColonAfterGroup(state))
        } else if c == '.' {
            Err(MachineExitStatus::Ipv4Trailer(state))
        } else {
            state.num_digits += 1;
            if state.num_digits > 4 {
                Err(Error::TooManyDigits.into())
            } else if DIGIT.contains(&c) {
                Ok(Self::InGroupCouldBeIpv4(state))
            } else if HEXDIG.contains(&c) {
                Ok(Self::InGroupNotIpv4(state))
            } else {
                Err(Error::IllegalCharacter(Context::Ipv6Address).into())
            }
        }
    }

    fn next_colon_after_group(state: Shared<'a>, i: usize, c: char) -> Result<Self, MachineExitStatus> {
        let mut state = state;
        if c == ':' {
            if state.double_colon_encountered {
                Err(Error::TooManyDoubleColons.into())
            } else {
                state.double_colon_encountered = true;
                Ok(Self::AfterDoubleColon(state))
            }
        } else if DIGIT.contains(&c) {
            state.potential_ipv4_address_start = i;
            state.num_digits += 1;
            Ok(Self::InGroupCouldBeIpv4(state))
        } else if HEXDIG.contains(&c) {
            state.num_digits += 1;
            Ok(Self::InGroupNotIpv4(state))
        } else {
            Err(Error::IllegalCharacter(Context::Ipv6Address).into())
        }
    }
}

pub fn validate_ipv6_address<T>(address: T) -> Result<(), Error>
    where T: AsRef<str>
{
    let address = address.as_ref();
    address
        .char_indices()
        .try_fold(State::new(address), |machine, (i, c)| {
            machine.next(i, c)
        })
        .or_else(|machine_exit_status| match machine_exit_status {
            MachineExitStatus::Ipv4Trailer(state) => Ok(State::InGroupIpv4(state)),
            MachineExitStatus::Error(error) => Err(error)
        })?
        .finalize()
}
