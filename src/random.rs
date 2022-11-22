pub trait RandomNumberEngine: PartialEq {
    /// Type of the generated values.
    type ResultType;
    /// Type of the generator state.
    type StateType;

    /// Sets the current state of the engine.
    fn seed(&mut self, seed: Option<Self::StateType>);
    /// Gets the generator state.
    fn state(&self) -> Self::StateType;
    /// Advances the engine's state and returns the generated value.
    fn get(&mut self) -> Self::ResultType;
    /// Advances the engine's state by \a z steps.
    fn discard(&mut self, z: usize);
    /// Gets the smallest possible value in the output range.
    const MIN: Self::ResultType;
    /// Gets the largest possible value in the output range.
    const MAX: Self::ResultType;
}

/// Random number engine for 32 bit random numbers.
/// This is a minimal implementation of the pcg32_fast PRNG from https://www.pcg-random.org.
/// The internal state has 64 bit, so there are no 128 bit operations required.
/// TODO: Implement 64 bit generator if required.
#[derive(Debug)]
pub struct PCG32Fast {
    state: <Self as RandomNumberEngine>::StateType,
}

impl RandomNumberEngine for PCG32Fast {
    type ResultType = u32;
    type StateType = u64;

    const MIN: Self::ResultType = Self::ResultType::MIN;
    const MAX: Self::ResultType = Self::ResultType::MAX;

    fn seed(&mut self, seed: Option<Self::StateType>) {
        self.state = Self::mangle_seed(seed);
    }

    fn state(&self) -> Self::StateType {
        self.state
    }

    fn get(&mut self) -> Self::ResultType {
        let old_state = self.state;
        self.advance();
        Self::output(old_state)
    }

    fn discard(&mut self, z: usize) {
        for _ in 0..z {
            self.advance();
        }
        // TODO: self.state *= integral_pow_overflow(Self::MULTIPLIER, z);
    }
}

// NOTE: This could be improved with inherent_associated_types which is not implemented yet.
impl PCG32Fast {
    const MULTIPLIER: u64 = 6364136223846793005;
    const DEFAULT_SEED: <PCG32Fast as RandomNumberEngine>::StateType = 0xcafef00dd15ea5e5;

    /// Constructs the engine with a default or optionally given seed.
    pub fn new(seed: Option<<PCG32Fast as RandomNumberEngine>::StateType>) -> Self {
        Self {
            state: Self::mangle_seed(seed),
        }
    }

    fn mangle_seed(
        seed: Option<<PCG32Fast as RandomNumberEngine>::StateType>,
    ) -> <PCG32Fast as RandomNumberEngine>::StateType {
        type StateType = <PCG32Fast as RandomNumberEngine>::StateType;
        seed.unwrap_or(Self::DEFAULT_SEED) | (3 as StateType)
    }

    fn advance(&mut self) {
        self.state = self.state.wrapping_mul(Self::MULTIPLIER);
    }

    // XSH RS -- high xorshift, followed by a random shift
    #[allow(clippy::int_plus_one, clippy::bool_to_int_with_if)]
    const fn output(
        state: <PCG32Fast as RandomNumberEngine>::StateType,
    ) -> <PCG32Fast as RandomNumberEngine>::ResultType {
        type ResultType = <PCG32Fast as RandomNumberEngine>::ResultType;
        type StateType = <PCG32Fast as RandomNumberEngine>::StateType;
        const BITS: u8 = StateType::BITS as u8;
        const XTYPEBITS: u8 = ResultType::BITS as u8;
        const SPAREBITS: u8 = BITS - XTYPEBITS;
        const OPBITS: u8 = if SPAREBITS - 5 >= 64 {
            5
        } else if SPAREBITS - 4 >= 32 {
            4
        } else if SPAREBITS - 3 >= 16 {
            3
        } else if SPAREBITS - 2 >= 4 {
            2
        } else if SPAREBITS - 1 >= 1 {
            1
        } else {
            0
        };
        const MASK: u8 = (1 << OPBITS) - 1;
        const MAXRANDSHIFT: u8 = MASK;
        const TOPSPARE: u8 = OPBITS;
        const BOTTOMSPARE: u8 = SPAREBITS - TOPSPARE;
        const XSHIFT: u8 = TOPSPARE + (XTYPEBITS + MAXRANDSHIFT) / 2;
        let rshift: u8 = if OPBITS != 0 {
            (state >> (BITS - OPBITS)) as u8 & MASK
        } else {
            0
        };
        ((state ^ (state >> XSHIFT)) >> (BOTTOMSPARE - MAXRANDSHIFT + rshift)) as ResultType
    }

    //fn wrapped(&self) -> bool {
    //    self.state == 3
    //}
}

impl std::cmp::PartialEq for PCG32Fast {
    /// Compares the internal states of two pseudo-random number engines.
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type ResultType = <PCG32Fast as RandomNumberEngine>::ResultType;

    // The first 32 numbers of the default seeded pcg32_fast reference implementation.
    const REF_NUMBERS: [ResultType; 32] = [
        0xafef3262, 0x1fa2dd91, 0xea48e0b1, 0xb61b4748, 0xa52ec9aa, 0x11a1d5d3, 0x1b91f38c,
        0xe3d4226a, 0x7061b6f3, 0x4dd9129d, 0xf7f7ab2e, 0x8b762307, 0x90622a87, 0x77f21803,
        0x2ad14e00, 0x848768d7, 0x1ea57d20, 0xee7df193, 0x37c7776c, 0x3b2b210e, 0xaa8babda,
        0xa08fa273, 0x71cbdd0a, 0x9493e3bb, 0xa33a0c63, 0x96fb42bf, 0x64fc0de, 0xd79cec24,
        0xb452ead, 0x215e88ca, 0x8e41e6b2, 0x38d506a5,
    ];

    #[test]
    fn match_pcg32_fast_reference_implementation() {
        let mut rng = PCG32Fast::new(None);
        let mut numbers: [ResultType; 32] = [0; 32];
        numbers.iter_mut().for_each(|x| *x = rng.get());
        assert_eq!(numbers, REF_NUMBERS);
    }

    #[test]
    fn random_number_engine_seed() {
        let mut rng = PCG32Fast::new(None);
        rng.discard(REF_NUMBERS.len() / 2);
        rng.seed(None);
        let mut numbers: [ResultType; 32] = [0; 32];
        numbers.iter_mut().for_each(|x| *x = rng.get());
        assert_eq!(numbers, REF_NUMBERS);
    }

    #[test]
    fn random_number_engine_discard() {
        let mut rng = PCG32Fast::new(None);
        const MIDDLE_I: usize = REF_NUMBERS.len() / 2;
        rng.discard(MIDDLE_I);
        REF_NUMBERS[MIDDLE_I..]
            .iter()
            .for_each(|x| assert_eq!(rng.get(), *x));
    }

    #[test]
    fn random_number_engine_partial_eq() {
        let mut rng1 = PCG32Fast::new(None);
        let rng2 = PCG32Fast::new(None);
        assert_eq!(rng1, rng2);
        rng1.discard(1);
        assert_ne!(rng1, rng2);
    }
}
