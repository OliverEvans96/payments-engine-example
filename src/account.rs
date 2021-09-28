use crate::types::{Account, Deposit, Withdrawal};

pub struct LockedAccount<'a>(&'a mut Account);
pub struct UnlockedAccount<'a>(&'a mut Account);

impl<'a> UnlockedAccount<'a> {
    /// Consumes the current access object and returns
    /// an access with downgraded permissions.
    /// NOTE: This doesn't actually lock the account,
    /// and only affects this access instance,
    /// not the account itself.
    fn downgrade(self) -> LockedAccount<'a> {
        LockedAccount(self.0)
    }
    // TODO: Rethink this
}

mod private {
    // A bit hacky, but this is a workaround to avoid exposing
    // WrapsAccount publicly (since we don't want to grant 
    // public access to the underlying account - that would 
    // kind of defeat the point of the wrapper).
    // Normally, it's a warning (soon-to-be error) to expose
    // a private trait (WrapsAccount) 
    // in a public interface (BaseAccountFeatures)
    // See https://github.com/rust-lang/rust/issues/34537
    use super::Account;
    pub trait WrapsAccount {
        fn get_account(&self) -> &Account;
        fn get_mut_account(&mut self) -> &mut Account;
    }
}

impl<'a> private::WrapsAccount for LockedAccount<'a> {
    #[inline]
    fn get_account(&self) -> &Account {
        &self.0
    }

    #[inline]
    fn get_mut_account(&mut self) -> &mut Account {
        &mut self.0
    }
}

impl<'a> private::WrapsAccount for UnlockedAccount<'a> {
    #[inline]
    fn get_account(&self) -> &Account {
        &self.0
    }

    #[inline]
    fn get_mut_account(&mut self) -> &mut Account {
        &mut self.0
    }
}

pub trait BaseAccountFeatures: private::WrapsAccount {
    fn modify_balances_for_dispute(&mut self, disputed_deposit: &Deposit) {
        self.get_mut_account().available -= disputed_deposit.amount;
        self.get_mut_account().held += disputed_deposit.amount;
    }
    fn modify_balances_for_resolve(&mut self, disputed_deposit: &Deposit) {
        self.get_mut_account().available += disputed_deposit.amount;
        self.get_mut_account().held -= disputed_deposit.amount;
    }
    fn modify_balances_for_chargeback(&mut self, disputed_deposit: &Deposit) {
        self.get_mut_account().held -= disputed_deposit.amount;
    }
    // TODO: Feels like this should be in UnlockedAccountFeatures
    fn lock(&mut self) {
        self.get_mut_account().locked = true;
    }
    fn view(&self) -> &Account {
        self.get_account()
    }
}

pub trait UnlockedAccountFeatures: private::WrapsAccount {
    fn modify_balances_for_deposit(&mut self, deposit: &Deposit) {
        self.get_mut_account().available += deposit.amount;
    }
    fn modify_balances_for_withdrawal(&mut self, withdrawal: &Withdrawal) {
        self.get_mut_account().available -= withdrawal.amount;
    }
}

impl<'a> BaseAccountFeatures for LockedAccount<'a> {}
impl<'a> BaseAccountFeatures for UnlockedAccount<'a> {}
impl<'a> UnlockedAccountFeatures for UnlockedAccount<'a> {}

impl Account {
    pub fn access<'a>(&'a mut self) -> AccountAccess<'a> {
        if self.locked {
            AccountAccess::Locked(LockedAccount(self))
        } else {
            AccountAccess::Unlocked(UnlockedAccount(self))
        }
    }
}
pub enum AccountAccess<'a> {
    Locked(LockedAccount<'a>),
    Unlocked(UnlockedAccount<'a>),
}

impl<'a> AccountAccess<'a> {
    /// Consume the access and return a reference to the contained
    /// account wrapper, providing only the base account features.
    pub fn inner(self) -> impl BaseAccountFeatures + 'a {
        match self {
            AccountAccess::Locked(account) => account,
            AccountAccess::Unlocked(account) => account.downgrade(),
        }
    }
}
