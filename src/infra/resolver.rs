use once_cell::sync::OnceCell;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// The target service type to be resolved by the resolver.
pub enum Target {
    REST,    // restful service
    GRPC,    // grpc service
    THRIFT,  // thrift service
    GRAPHQL, // graphql service
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::REST => write!(f, "rest"),
            Target::GRPC => write!(f, "grpc"),
            Target::THRIFT => write!(f, "thrift"),
            Target::GRAPHQL => write!(f, "graphql"),
        }
    }
}

/// Register grabbed a closure for generating values without
/// use static block to define a value.
#[derive(Clone)]
pub struct Register<T>(Arc<dyn Fn() -> T + Send + Sync>);

impl<T> Register<T> {
    pub fn once(f: impl Fn() -> T + Send + Sync + 'static) -> Self
    where
        T: Send + Sync + Clone + 'static,
    {
        let cell = OnceCell::new();
        Register(Arc::new(move || cell.get_or_init(|| f()).clone()))
    }

    /// Use Box::leak to create a 'static lifetime register.
    /// Used for high performance scenarios, for normal scenarios please use [Register::once]
    /// Keep in mind that the return type T will be leaked in the memory, so
    /// DO NOT call this in recurrent block.
    pub fn once_ref(f: impl Fn() -> T + Send + Sync + 'static) -> Register<&'static T>
    where
        T: Sync + 'static,
    {
        let cell = OnceCell::new();
        Register(Arc::new(move || {
            cell.get_or_init(|| Box::leak(Box::new(f())) as &'static T)
        }))
    }

    /// Create a register that returns a new instance of a value each time.
    pub fn factory(f: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Register(Arc::new(f))
    }
}

pub trait BaseResolver {
    // The target service type to be resolved by the resolver.
    const TARGET: Target;

    /// Resolve a register.
    fn resolve<T>(&self, register: &Register<T>) -> T {
        register.0()
    }
}

pub trait NamedResolver: BaseResolver {
    // The service id in the whole system
    const SID: &'static str;
}
