use crate::traits::Disputable;
use crate::types::Account;
use crate::types::{Deposit, Withdrawal};

/// A locked account cannot deposit or withdraw.
pub struct LockedAccount<'a>(&'a mut Account);

/// An unlocked account can deposit or withdraw.
pub struct UnlockedAccount<'a>(&'a mut Account);

impl Account {
    /// Get appropriate mutable access into the account
    /// based on its state (locked or unlocked).
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

    /// Marker trait for a type that privately holds an Account,
    /// but does not necessarily expose it publicly.
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

/// This trait implements functionality common to all accounts,
/// namely viewing, disputing, resolving, and charging back.
pub trait BaseAccountFeatures: private::WrapsAccount {
    // Since we're using this trait as an object somewhere,
    // these functions can only use dynamic dispatch.
    // They can't be generic over traits.
    // See https://doc.rust-lang.org/reference/items/traits.html#object-safety
    // TODO: Remove this & undo dyns?
    fn modify_balances_for_dispute(&mut self, disputed_tx: &dyn Disputable) {
        let account = self.get_mut_account();
        disputed_tx.modify_balances_for_dispute(account);
    }
    fn modify_balances_for_resolve(&mut self, resolved_tx: &dyn Disputable) {
        let account = self.get_mut_account();
        resolved_tx.modify_balances_for_resolve(account);
    }
    fn modify_balances_for_chargeback(&mut self, chargebackd_tx: &dyn Disputable) {
        let account = self.get_mut_account();
        chargebackd_tx.modify_balances_for_chargeback(account);
    }

    fn view(&self) -> &Account {
        self.get_account()
    }
}

/// Only unlocked accounts may deposit, withdraw, or lock.
pub trait UnlockedAccountFeatures: private::WrapsAccount {
    fn modify_balances_for_deposit(&mut self, deposit: &Deposit) {
        self.get_mut_account().available += deposit.amount;
    }
    fn modify_balances_for_withdrawal(&mut self, withdrawal: &Withdrawal) {
        self.get_mut_account().available -= withdrawal.amount;
    }
    fn lock(&mut self) {
        self.get_mut_account().locked = true;
    }
}

impl<'a> BaseAccountFeatures for LockedAccount<'a> {}
impl<'a> BaseAccountFeatures for UnlockedAccount<'a> {}
impl<'a> UnlockedAccountFeatures for UnlockedAccount<'a> {}

impl<'a> AccountAccess<'a> {
    /// Consume the access and return a reference to the contained
    /// account wrapper, providing only the base account features.
    pub fn inner(self) -> Box<dyn BaseAccountFeatures + 'a> {
        match self {
            AccountAccess::Locked(account) => Box::new(account),
            AccountAccess::Unlocked(account) => Box::new(account),
        }
    }
}

impl<'a> private::WrapsAccount for AccountAccess<'a> {
    fn get_account(&self) -> &Account {
        match self {
            AccountAccess::Locked(account) => account.get_account(),
            AccountAccess::Unlocked(account) => account.get_account(),
        }
    }
    fn get_mut_account(&mut self) -> &mut Account {
        match self {
            AccountAccess::Locked(account) => account.get_mut_account(),
            AccountAccess::Unlocked(account) => account.get_mut_account(),
        }
    }
}

impl<'a> BaseAccountFeatures for AccountAccess<'a> {}

#[cfg(test)]
mod tests {
    use crate::account::{AccountAccess, UnlockedAccountFeatures};
    use crate::types::Account;

    #[test]
    fn test_account_unlocked() {
        let mut account = Account::default();
        assert!(matches!(account.access(), AccountAccess::Unlocked(_)));
    }

    #[test]
    fn test_account_locked() {
        let mut account = Account::default();
        account.locked = true;
        assert!(matches!(account.access(), AccountAccess::Locked(_)));
    }

    #[test]
    fn test_lock_account() {
        let mut account = Account::default();
        if let AccountAccess::Unlocked(mut access) = account.access() {
            access.lock();
        } else {
            assert!(false);
        }
        assert!(matches!(account.access(), AccountAccess::Locked(_)));
        assert_eq!(account.locked, true);
    }
}
