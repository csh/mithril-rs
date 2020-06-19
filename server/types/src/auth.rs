pub trait AuthenticationStrategy {
    fn authenticate(&self, username: String, password: String) -> anyhow::Result<bool>;
}

pub struct Authenticator {
    strategy: Box<dyn AuthenticationStrategy + Send + Sync + 'static>,
}

impl Authenticator {
    pub fn new<A: AuthenticationStrategy + Send + Sync + 'static>(strategy: A) -> Self {
        Authenticator {
            strategy: Box::new(strategy),
        }
    }

    pub fn authenticate(&self, username: String, password: String) -> anyhow::Result<bool> {
        self.strategy.authenticate(username, password)
    }
}

pub struct AlwaysAllowStrategy;

impl AuthenticationStrategy for AlwaysAllowStrategy {
    fn authenticate(&self, _username: String, _password: String) -> anyhow::Result<bool> {
        Ok(true)
    }
}
