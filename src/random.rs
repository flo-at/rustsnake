pub trait RandomNumberEngine {
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

pub use pcg32fast::PCG32Fast;

/// Random number engine for 32 bit random numbers.
/// This is a minimal implementation of the pcg32_fast PRNG from https://www.pcg-random.org.
/// The internal state has 64 bit, so there are no 128 bit operations required.
/// Fulfills the std::uniform_random_bit_generator concept but not the RandomNumberEngine C++ named requirement.
/// @todo Die unteren beiden bits im seed werden ignoriert wenn ich das richtig verstehe (bleiben 62 bits). Hierfür muss
///       ggf. ein check her.
/// @todo Implement 64 bit generator if required.
/// @see https://www.pcg-random.org
mod pcg32fast {

    type ResultType = u32;
    type StateType = u64;

    pub struct PCG32Fast {
        state: StateType,
    }

    impl super::RandomNumberEngine for PCG32Fast {
        type ResultType = ResultType;
        type StateType = StateType;

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
            //self.state *= ::masheen::utils::helpers::integral_pow_overflow(Self::MULTIPLIER, z);
            // TODO
        }
    }

    impl PCG32Fast {
        const MULTIPLIER: u64 = 6364136223846793005;
        /// Default generator seed.
        const DEFAULT_SEED: StateType = 0xcafef00dd15ea5e5;

        /// Constructs the engine with a default or optionally given seed.
        pub fn new(seed: Option<StateType>) -> Self {
            Self {
                state: Self::mangle_seed(seed),
            }
        }

        fn mangle_seed(seed: Option<StateType>) -> StateType {
            seed.unwrap_or(Self::DEFAULT_SEED) | (3 as StateType)
        }

        fn advance(&mut self) {
            self.state = self.state.wrapping_mul(Self::MULTIPLIER);
        }

        // XSH RS -- high xorshift, followed by a random shift
        const fn output(state: StateType) -> ResultType {
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
    }

    impl std::cmp::PartialEq for PCG32Fast {
        /// Compares the internal states of two pseudo-random number engines.
        fn eq(&self, other: &Self) -> bool {
            self.state == other.state
        }
    }

    // bool wrapped() const noexcept {
    //     return state == 3;
    // }
}
