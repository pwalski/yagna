use crate::dao::{activity, agreement, allocation};
use crate::error::DbResult;
use crate::models::payment::{
    ActivityPayment as DbActivityPayment, AgreementPayment as DbAgreementPayment, ReadObj, WriteObj,
};
use crate::schema::pay_activity_payment::dsl as activity_pay_dsl;
use crate::schema::pay_agreement_payment::dsl as agreement_pay_dsl;
use crate::schema::pay_payment::dsl;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl,
};
use std::collections::HashMap;
use ya_client_model::payment::{ActivityPayment, AgreementPayment, Payment};
use ya_client_model::NodeId;
use ya_persistence::executor::{
    do_with_transaction, readonly_transaction, AsDao, ConnType, PoolType,
};
use ya_persistence::types::Role;

pub struct PaymentDao<'c> {
    pool: &'c PoolType,
}

fn insert_activity_payments(
    activity_payments: Vec<ActivityPayment>,
    payment_id: &String,
    owner_id: &NodeId,
    conn: &ConnType,
) -> DbResult<()> {
    log::trace!("Inserting activity payments...");
    for activity_payment in activity_payments {
        let amount = activity_payment.amount.into();
        activity::increase_amount_paid(&activity_payment.activity_id, &owner_id, &amount, conn)?;
        diesel::insert_into(activity_pay_dsl::pay_activity_payment)
            .values(DbActivityPayment {
                payment_id: payment_id.clone(),
                activity_id: activity_payment.activity_id,
                owner_id: owner_id.clone(),
                amount,
            })
            .execute(conn)
            .map(|_| ())?;
    }
    log::trace!("Activity payments inserted.");
    Ok(())
}

fn insert_agreement_payments(
    agreement_payments: Vec<AgreementPayment>,
    payment_id: &String,
    owner_id: &NodeId,
    conn: &ConnType,
) -> DbResult<()> {
    log::trace!("Inserting agreement payments...");
    for agreement_payment in agreement_payments {
        let amount = agreement_payment.amount.into();
        agreement::increase_amount_paid(&agreement_payment.agreement_id, &owner_id, &amount, conn)?;
        diesel::insert_into(agreement_pay_dsl::pay_agreement_payment)
            .values(DbAgreementPayment {
                payment_id: payment_id.clone(),
                agreement_id: agreement_payment.agreement_id,
                owner_id: owner_id.clone(),
                amount,
            })
            .execute(conn)
            .map(|_| ())?;
    }
    log::trace!("Agreement payments inserted.");
    Ok(())
}

impl<'c> AsDao<'c> for PaymentDao<'c> {
    fn as_dao(pool: &'c PoolType) -> Self {
        Self { pool }
    }
}

impl<'c> PaymentDao<'c> {
    async fn insert(
        &self,
        payment: WriteObj,
        activity_payments: Vec<ActivityPayment>,
        agreement_payments: Vec<AgreementPayment>,
    ) -> DbResult<()> {
        let payment_id = payment.id.clone();
        let owner_id = payment.owner_id.clone();
        let allocation_id = payment.allocation_id.clone();
        let amount = payment.amount.clone();

        do_with_transaction(self.pool, move |conn| {
            log::trace!("Inserting payment...");
            diesel::insert_into(dsl::pay_payment)
                .values(payment)
                .execute(conn)?;
            log::trace!("Payment inserted.");

            insert_activity_payments(activity_payments, &payment_id, &owner_id, &conn)?;
            insert_agreement_payments(agreement_payments, &payment_id, &owner_id, &conn)?;

            // Update spent & remaining amount for allocation (if applicable)
            if let Some(allocation_id) = &allocation_id {
                log::trace!("Updating spent & remaining amount for allocation...");
                allocation::spend_from_allocation(allocation_id, &amount, conn)?;
                log::trace!("Allocation updated.");
            }

            Ok(())
        })
        .await
    }

    pub async fn create_new(
        &self,
        payer_id: NodeId,
        payee_id: NodeId,
        payer_addr: String,
        payee_addr: String,
        allocation_id: String,
        amount: BigDecimal,
        details: Vec<u8>,
        activity_payments: Vec<ActivityPayment>,
        agreement_payments: Vec<AgreementPayment>,
    ) -> DbResult<String> {
        let payment = WriteObj::new_sent(
            payer_id,
            payee_id,
            payer_addr,
            payee_addr,
            allocation_id,
            amount,
            details,
        );
        let payment_id = payment.id.clone();
        self.insert(payment, activity_payments, agreement_payments)
            .await?;
        Ok(payment_id)
    }

    pub async fn insert_received(&self, payment: Payment, payee_id: NodeId) -> DbResult<()> {
        let activity_payments = payment.activity_payments.clone();
        let agreement_payments = payment.agreement_payments.clone();
        let payment = WriteObj::new_received(payment);
        self.insert(payment, activity_payments, agreement_payments)
            .await
    }

    pub async fn get(&self, payment_id: String, owner_id: NodeId) -> DbResult<Option<Payment>> {
        readonly_transaction(self.pool, move |conn| {
            let payment: Option<ReadObj> = dsl::pay_payment
                .filter(dsl::id.eq(&payment_id))
                .filter(dsl::owner_id.eq(&owner_id))
                .first(conn)
                .optional()?;

            match payment {
                Some(payment) => {
                    let activity_payments = activity_pay_dsl::pay_activity_payment
                        .filter(activity_pay_dsl::payment_id.eq(&payment_id))
                        .filter(activity_pay_dsl::owner_id.eq(&owner_id))
                        .load(conn)?;
                    let agreement_payments = agreement_pay_dsl::pay_agreement_payment
                        .filter(agreement_pay_dsl::payment_id.eq(&payment_id))
                        .filter(agreement_pay_dsl::owner_id.eq(&owner_id))
                        .load(conn)?;
                    Ok(Some(
                        payment.into_api_model(activity_payments, agreement_payments),
                    ))
                }
                None => Ok(None),
            }
        })
        .await
    }

    async fn get_for_role(
        &self,
        node_id: NodeId,
        later_than: Option<NaiveDateTime>,
        role: Role,
    ) -> DbResult<Vec<Payment>> {
        readonly_transaction(self.pool, move |conn| {
            let query = dsl::pay_payment
                .filter(dsl::owner_id.eq(&node_id))
                .filter(dsl::role.eq(&role))
                .order_by(dsl::timestamp.asc());
            let payments: Vec<ReadObj> = match later_than {
                Some(timestamp) => query.filter(dsl::timestamp.gt(timestamp)).load(conn)?,
                None => query.load(conn)?,
            };
            let activity_payments = activity_pay_dsl::pay_activity_payment
                .inner_join(
                    dsl::pay_payment.on(activity_pay_dsl::owner_id
                        .eq(dsl::owner_id)
                        .and(activity_pay_dsl::payment_id.eq(dsl::id))),
                )
                .filter(dsl::owner_id.eq(&node_id))
                .filter(dsl::role.eq(&role))
                .select(crate::schema::pay_activity_payment::all_columns)
                .load(conn)?;
            let agreement_payments = agreement_pay_dsl::pay_agreement_payment
                .inner_join(
                    dsl::pay_payment.on(agreement_pay_dsl::owner_id
                        .eq(dsl::owner_id)
                        .and(agreement_pay_dsl::payment_id.eq(dsl::id))),
                )
                .filter(dsl::owner_id.eq(&node_id))
                .filter(dsl::role.eq(&role))
                .select(crate::schema::pay_agreement_payment::all_columns)
                .load(conn)?;
            Ok(join_activity_and_agreement_payments(
                payments,
                activity_payments,
                agreement_payments,
            ))
        })
        .await
    }

    pub async fn get_for_requestor(
        &self,
        node_id: NodeId,
        later_than: Option<NaiveDateTime>,
    ) -> DbResult<Vec<Payment>> {
        self.get_for_role(node_id, later_than, Role::Requestor)
            .await
    }

    pub async fn get_for_provider(
        &self,
        node_id: NodeId,
        later_than: Option<NaiveDateTime>,
    ) -> DbResult<Vec<Payment>> {
        self.get_for_role(node_id, later_than, Role::Provider).await
    }
}

fn join_activity_and_agreement_payments(
    payments: Vec<ReadObj>,
    activity_payments: Vec<DbActivityPayment>,
    agreement_payments: Vec<DbAgreementPayment>,
) -> Vec<Payment> {
    let mut activity_payments_map =
        activity_payments
            .into_iter()
            .fold(HashMap::new(), |mut map, activity_payment| {
                map.entry(activity_payment.payment_id.clone())
                    .or_insert_with(Vec::new)
                    .push(activity_payment);
                map
            });
    let mut agreement_payments_map =
        agreement_payments
            .into_iter()
            .fold(HashMap::new(), |mut map, agreement_payment| {
                map.entry(agreement_payment.payment_id.clone())
                    .or_insert_with(Vec::new)
                    .push(agreement_payment);
                map
            });
    payments
        .into_iter()
        .map(|payment| {
            let activity_payments = activity_payments_map.remove(&payment.id).unwrap_or(vec![]);
            let agreement_payments = agreement_payments_map.remove(&payment.id).unwrap_or(vec![]);
            payment.into_api_model(activity_payments, agreement_payments)
        })
        .collect()
}
