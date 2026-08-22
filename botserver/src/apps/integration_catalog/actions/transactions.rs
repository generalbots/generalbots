use super::super::types::{ActionTemplate, Parameter, ParameterType};
use super::{
    destructive, param, read, write, CREATE_PARAMS, DELETE_PARAMS, GET_PARAMS, LIST_PARAMS,
    SEARCH_PARAMS,
};

const REPORT_RANGE: &[Parameter] = &[
    param("start", ParameterType::DateTime, false, "Range start"),
    param("end", ParameterType::DateTime, false, "Range end"),
];
const TRANSFER_CREATE: &[Parameter] = &[
    param(
        "destination",
        ParameterType::String,
        true,
        "Destination account",
    ),
    param(
        "amount",
        ParameterType::String,
        true,
        "Transfer amount and currency",
    ),
    param(
        "reference",
        ParameterType::String,
        false,
        "Transfer reference",
    ),
];
const TRACK_SHIPMENT: &[Parameter] = &[param(
    "tracking_number",
    ParameterType::String,
    true,
    "Carrier tracking number",
)];
const REFUND_PAYMENT: &[Parameter] = &[
    param(
        "payment_id",
        ParameterType::String,
        true,
        "Payment identifier",
    ),
    param(
        "amount",
        ParameterType::String,
        false,
        "Optional partial refund amount",
    ),
    param("reason", ParameterType::String, false, "Refund reason"),
];
const QUOTE_PARAMS: &[Parameter] = &[
    param(
        "base",
        ParameterType::String,
        true,
        "Base asset or currency",
    ),
    param(
        "quote",
        ParameterType::String,
        true,
        "Quote asset or currency",
    ),
];

pub(crate) const SHIPPING_ACTIONS: &[ActionTemplate] = &[
    read(
        "shipments.list",
        "list",
        "List shipments",
        "List shipments.",
        LIST_PARAMS,
    ),
    read(
        "shipments.track",
        "track",
        "Track shipment",
        "Track a shipment.",
        TRACK_SHIPMENT,
    ),
    read(
        "rates.get",
        "get",
        "Get rates",
        "Get shipping rates.",
        CREATE_PARAMS,
    ),
    write(
        "shipments.create",
        "create",
        "Create shipment",
        "Create a shipment and label.",
        CREATE_PARAMS,
    ),
    destructive(
        "shipments.cancel",
        "cancel",
        "Cancel shipment",
        "Cancel a shipment.",
        DELETE_PARAMS,
    ),
];

pub(crate) const PAYMENT_ACTIONS: &[ActionTemplate] = &[
    read(
        "payments.list",
        "list",
        "List payments",
        "List payments.",
        LIST_PARAMS,
    ),
    read(
        "payments.get",
        "get",
        "Get payment",
        "Read payment details.",
        GET_PARAMS,
    ),
    read(
        "customers.search",
        "search",
        "Search customers",
        "Search payment customers.",
        SEARCH_PARAMS,
    ),
    write(
        "payments.create",
        "create",
        "Create payment",
        "Create or authorize a payment.",
        CREATE_PARAMS,
    ),
    destructive(
        "payments.refund",
        "refund",
        "Refund payment",
        "Refund a payment.",
        REFUND_PAYMENT,
    ),
];

// Stripe concrete action profile (#950 slice 2): PAYMENT_ACTIONS stays
// generic for providers without a live adapter, while Stripe carries its own
// keys mirroring botintegrations::providers::stripe::STRIPE_IMPLEMENTED_ACTIONS
// exactly.
const STRIPE_REFUND: &[Parameter] = &[
    param(
        "payment_intent",
        ParameterType::String,
        true,
        "Payment intent identifier",
    ),
    param(
        "amount",
        ParameterType::Integer,
        false,
        "Optional partial refund amount",
    ),
    param("reason", ParameterType::String, false, "Refund reason"),
];
const NO_PARAMS: &[Parameter] = &[];

pub(crate) const STRIPE_ACTIONS: &[ActionTemplate] = &[
    read(
        "balance.retrieve",
        "get",
        "Get balance",
        "Read the Stripe account balance.",
        NO_PARAMS,
    ),
    read(
        "customers.list",
        "list",
        "List customers",
        "List payment customers.",
        LIST_PARAMS,
    ),
    read(
        "customers.search",
        "search",
        "Search customers",
        "Search payment customers.",
        SEARCH_PARAMS,
    ),
    write(
        "customers.create",
        "create",
        "Create customer",
        "Create a payment customer.",
        CREATE_PARAMS,
    ),
    read(
        "payment_intents.list",
        "list",
        "List payments",
        "List payments.",
        LIST_PARAMS,
    ),
    read(
        "payment_intents.get",
        "get",
        "Get payment",
        "Read payment details.",
        GET_PARAMS,
    ),
    write(
        "payment_intents.create",
        "create",
        "Create payment",
        "Create or authorize a payment.",
        CREATE_PARAMS,
    ),
    read(
        "prices.list",
        "list",
        "List prices",
        "List active prices.",
        LIST_PARAMS,
    ),
    destructive(
        "refunds.create",
        "create",
        "Refund payment",
        "Refund a payment.",
        STRIPE_REFUND,
    ),
    read(
        "subscriptions.list",
        "list",
        "List subscriptions",
        "List subscriptions.",
        LIST_PARAMS,
    ),
];

pub(crate) const CRYPTO_ACTIONS: &[ActionTemplate] = &[
    read(
        "accounts.list",
        "list",
        "List accounts",
        "List crypto accounts or wallets.",
        LIST_PARAMS,
    ),
    read(
        "transactions.list",
        "list",
        "List transactions",
        "List crypto transactions.",
        LIST_PARAMS,
    ),
    read(
        "quotes.get",
        "get",
        "Get quote",
        "Get an asset price quote.",
        QUOTE_PARAMS,
    ),
    write(
        "transfers.create",
        "create",
        "Create transfer",
        "Create a crypto transfer.",
        TRANSFER_CREATE,
    ),
    destructive(
        "orders.cancel",
        "cancel",
        "Cancel order",
        "Cancel an open crypto order.",
        DELETE_PARAMS,
    ),
];

pub(crate) const BANKING_ACTIONS: &[ActionTemplate] = &[
    read(
        "accounts.list",
        "list",
        "List accounts",
        "List financial accounts.",
        LIST_PARAMS,
    ),
    read(
        "transactions.list",
        "list",
        "List transactions",
        "List account transactions.",
        REPORT_RANGE,
    ),
    read(
        "transactions.search",
        "search",
        "Search transactions",
        "Search account transactions.",
        SEARCH_PARAMS,
    ),
    read(
        "balances.get",
        "get",
        "Get balance",
        "Read an account balance.",
        GET_PARAMS,
    ),
    write(
        "transfers.create",
        "create",
        "Create transfer",
        "Create a bank transfer.",
        TRANSFER_CREATE,
    ),
];
