use scrypto::prelude::*;

// Issuer(s) are identified by their company legal entity identifier (LEI) issued by GLEIF Foundation
#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct IssuerBadge {
    #[mutable]
    company_lei: String,
}

// Issuer Agent is appointed by Issuer and linked through the issuer's badge id's
#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct IssuerAgentBadge {
    issuer_badge_id: ResourceAddress,
    issuer_badge_local_id: NonFungibleLocalId,
    #[mutable]
    company_lei: String,
}

// Simplified Investor Badge recording the country of residence
#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct InvestorBadge {
    country: String,
}

// Records the terms of the escrow as defined during the security subscription process, used in Subscription NFT
#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct SubscriptionEscrowTerms {
    party: String,
    symbol: String,
    rec_resource: ResourceAddress,
    rec_qty: Decimal,
    pay_resource: ResourceAddress,
    pay_amount: Decimal,
    #[mutable]
    escrow_status: String,
}

// Records the corporate actions related to the Security Instrument, used in Instrument NFT
#[derive(ScryptoSbor, NonFungibleData)]
struct InstrumentLifecycleData {
    action_type: String,
    percent: Decimal,
    #[mutable]
    available: bool,
}


#[blueprint]
mod securities_manager {
    enable_method_auth! {
        roles {
            issuer => updatable_by: [OWNER];
            issuer_agent => updatable_by: [OWNER];
            investor => updatable_by: [OWNER];
        },
        methods {
             issuer_mint_agent_badge => restrict_to:[issuer];
             issuer_create_instrument => restrict_to:[issuer];
             issuer_open_subscription => restrict_to:[issuer];
             issuer_close_subscription => restrict_to:[issuer];
             issuer_update_instrument_metadata => restrict_to:[issuer];
             issuer_claim_cash => restrict_to:[issuer];
             issuer_deposit_funds => restrict_to:[issuer];
             agent_add_instrument_lifecycle => restrict_to:[issuer_agent];
             agent_issue_lifecycle_securities => restrict_to:[issuer_agent];
             investor_check_kyc => PUBLIC;
             investor_subscribe => restrict_to:[investor];
             investor_transfer_payment => restrict_to:[investor];
             investor_cancel_payment => restrict_to:[investor];
             investor_claim_security => restrict_to:[investor];
             investor_claim_corporate_action => restrict_to:[investor];
             get_instruments => PUBLIC;
        }
    }

    struct SecuritiesManager {
        // instrument refers to the static data of the security instrument
        // subscription refers to the data of the security subscription process
        // security holdings refers to the fungible asset representation of the instrument static data
        owner_badge: ResourceAddress,
        system_badge: ResourceAddress,
        system_badge_vault: FungibleVault,
        issuer_badge_manager: ResourceManager,
        issuer_agent_badge_manager: ResourceManager,
        investor_badge_manager: ResourceManager,
        instrument_manager: Vec<ResourceManager>,
        instrument_vault: HashMap<ResourceAddress, NonFungibleVault>, // mapping of an instrument nft resource address and nstrument nft vault
        instrument_version: HashMap<ResourceAddress, u64>, // mapping of an instrument nft resource address and the last id (used to derive the next local id of an nft)
        instrument_lifecycle: HashMap<NonFungibleGlobalId, NonFungibleGlobalId>, // mapping of the global id of the nft to the next global id in sequence (used to order the sequence of the lifecycle actions to be applied)
        subscription_manager: ResourceManager,
        subscription_manager_vault: NonFungibleVault,
        subscribed_amount: Decimal, //represents the running subscription total, subscriptions need to be run sequentially for now for any instrument
        security_holdings_manager: HashMap<ResourceAddress, ResourceAddress>, // mapping of the fungible security resource address to the instrument (or static data) resource adddress
        security_holdings_vault: HashMap<NonFungibleGlobalId, FungibleVault>, // mapping of the instrument global id or lifecycle event and the vault holding the related version of the fungible securities
        cash_holding_vault: FungibleVault, // cash vault holding the issuance proceeds and from where cash corporate actions are paid from
    }

    impl SecuritiesManager {
            // create a new Securities Manager component
        pub fn instantiate_securities_manager() -> (Global<SecuritiesManager>, FungibleBucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(SecuritiesManager::blueprint_id());

                // owner manages the platform which allows security issuers, issuer agents and investors to interact
            let owner_badge: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Securities Manager Owner Badge", locked;
                        "symbol" => "OWNER", locked;
                    }
                ))
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();

                // Create system badge to be used for authorization inside method calls
                // Stored in a vault and cannot be withdrawn unless Owner updates this rule
            let system_badge: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "System Badge", locked;
                        "symbol" => "SYSTEM", locked;
                    }
                ))
                .withdraw_roles(withdraw_roles! {
                    withdrawer => rule!(deny_all);
                    withdrawer_updater => OWNER;
                })
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();

                // Owner is responsible for onboarding Issuer(s)
                // Essentially Soulbound for the Issuer, Owner Badge is required to transfer this badge
                // Owner can recall this badge e.g. if inadvertently transferred by owner to a non issuer account
                // Badge can never be burned as is used in metadata of instruments to map an issuer to an instrument
                // Manifest -> 02_owner_mint_issuer_badge.rtm
            let issuer_badge_manager = ResourceBuilder::new_integer_non_fungible::<IssuerBadge>(
                OwnerRole::Fixed(rule!(require(owner_badge.resource_address()))),
            )
            .metadata(metadata!(
                init {
                    "name" => "Issuer Badge", locked;
                    "symbol" => "ISSUER", locked;
                }
            ))
            .mint_roles(mint_roles! {
                minter => OWNER;
                minter_updater => OWNER;
            })
            .withdraw_roles(withdraw_roles! {
                withdrawer => OWNER;
                withdrawer_updater => OWNER;
            })
            .recall_roles(recall_roles! {
                recaller => OWNER;
                recaller_updater => OWNER;
            })
            .burn_roles(burn_roles! {
                burner => rule!(deny_all);
                burner_updater => rule!(deny_all);
            })
            .create_with_no_initial_supply();

                // Issuers are responsible for appointing Issuer Agent(s)
                // Soulbound for Issuer Agent, only the Issuer can transfer this badge
                // Issuer can recall, burn, freeze this badge from the Issuer Agent
                // Mint roles allow issuer badge to mint using manifest, preferred approach is to use the method issuer_mint_agent_badge with the system badge
            let issuer_agent_badge_manager =
                ResourceBuilder::new_integer_non_fungible::<IssuerAgentBadge>(OwnerRole::Fixed(
                    rule!(require(issuer_badge_manager.address())),
                ))
                .metadata(metadata!(
                    roles {
                    metadata_setter => OWNER;
                    metadata_setter_updater => rule!(require(owner_badge.resource_address()));
                    metadata_locker => OWNER;
                    metadata_locker_updater => rule!(require(owner_badge.resource_address()));
                },
                    init {
                        "name" => "Agent Badge", locked;
                        "symbol" => "AGENT", locked;
                    }
                ))
                .mint_roles(mint_roles! {
                    minter => rule!(
                        require(issuer_badge_manager.address()) ||
                        require(system_badge.resource_address())
                    );
                    minter_updater => rule!(require(owner_badge.resource_address()));
                })
                .withdraw_roles(withdraw_roles! {
                    withdrawer => OWNER;
                    withdrawer_updater => rule!(require(owner_badge.resource_address()));
                })
                .recall_roles(recall_roles! {
                    recaller => OWNER;
                    recaller_updater => rule!(require(owner_badge.resource_address()));
                })
                .burn_roles(burn_roles! {
                    burner => OWNER;
                    burner_updater => rule!(require(owner_badge.resource_address()));
                })
                .freeze_roles(freeze_roles! {
                    freezer => OWNER;
                    freezer_updater => rule!(require(owner_badge.resource_address()));
                })
                .create_with_no_initial_supply();

                // Issuer(s) are responsible for onboarding investors
                // Minting is facilitated by a method call from the Investor passing a KYC Check
                // Demonstrates the Virtual Badge Pattern using global_caller
                // KYC Badge is soulbound for the Investor
                // Issuer can burn & recall Investor Badges
            let investor_badge_manager =
                ResourceBuilder::new_integer_non_fungible::<InvestorBadge>(OwnerRole::Fixed(
                    rule!(require(issuer_badge_manager.address())),
                ))
                .metadata(metadata!(
                    init {
                        "name" => "Investor KYC Badge", locked;
                        "symbol" => "INVESTOR", locked;
                    }
                ))
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                })
                .withdraw_roles(withdraw_roles! {
                    withdrawer => rule!(deny_all);
                    withdrawer_updater => rule!(require(owner_badge.resource_address()));
                })
                .recall_roles(recall_roles! {
                    recaller => OWNER;
                    recaller_updater => rule!(require(owner_badge.resource_address()));
                })
                .burn_roles(burn_roles! {
                    burner => OWNER;
                    burner_updater => rule!(require(owner_badge.resource_address()));
                })
                .create_with_no_initial_supply();

                // Issuer is responsible for managing the subscription process
                // Subscription process is an interactive process and uses the Virtual Badge Patterm for minting,
                // burning as well as updating the NFT data representing the Esrows terms.
            let subscription_manager =
                ResourceBuilder::new_integer_non_fungible::<SubscriptionEscrowTerms>(
                    OwnerRole::Fixed(rule!(require(issuer_badge_manager.address()))),
                )
                .metadata(metadata!(
                    roles {
                        metadata_setter => OWNER;
                        metadata_setter_updater => rule!(require(owner_badge.resource_address()));
                        metadata_locker => OWNER;
                        metadata_locker_updater => rule!(require(owner_badge.resource_address()));
                    },
                    init {
                        "name" => "Subscription Badge", locked;
                        "symbol" => "SUBSCRIPTION", locked;
                    }
                ))
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(require(owner_badge.resource_address()));
                })
                .burn_roles(burn_roles! {
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(require(owner_badge.resource_address()));
                })
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(global_caller(component_address)));
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .create_with_no_initial_supply();

                // initializes the data structures representing the instrument definitions and the related security assets
                // platform provides multi-instrument and multi-security asset support and structures are empty to begin with
            let instrument_manager: Vec<ResourceManager> = Vec::new();
            let instrument_version: HashMap<ResourceAddress, u64> = HashMap::new();
            let instrument_vault: HashMap<ResourceAddress, NonFungibleVault> = HashMap::new();
            let instrument_lifecycle: HashMap<NonFungibleGlobalId, NonFungibleGlobalId> =
                HashMap::new();
            let security_holdings_manager: HashMap<ResourceAddress, ResourceAddress> =
                HashMap::new();
            let security_holdings_vault: HashMap<NonFungibleGlobalId, FungibleVault> =
                HashMap::new();

            let component = Self {
                owner_badge: owner_badge.resource_address(),
                system_badge: system_badge.resource_address(),
                system_badge_vault: FungibleVault::with_bucket(system_badge),
                issuer_badge_manager: issuer_badge_manager,
                issuer_agent_badge_manager: issuer_agent_badge_manager,
                investor_badge_manager: investor_badge_manager,
                instrument_manager: instrument_manager,
                instrument_vault: instrument_vault,
                instrument_version: instrument_version,
                instrument_lifecycle: instrument_lifecycle,
                subscription_manager: subscription_manager,
                subscription_manager_vault: NonFungibleVault::new(subscription_manager.address()),
                subscribed_amount: dec!("0"),
                security_holdings_manager: security_holdings_manager,
                security_holdings_vault: security_holdings_vault,
                // for now we model cash using XRD
                cash_holding_vault: FungibleVault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                owner_badge.resource_address()
            ))))
            .roles(roles!(
                issuer => rule!(require(issuer_badge_manager.address()));
                issuer_agent => rule!(require(issuer_agent_badge_manager.address()));
                investor => rule!(require(investor_badge_manager.address()));
            ))
            .with_address(address_reservation)
            .globalize();

            (component, owner_badge)
        }

            // implemented as a method to ensure the issuer badge details are assigned to the issuer agent badge data programatically
            // reads the issuer data on the passed in issuer proof to be added to the issuer agent badge
            // authorization to mint provided by the system badge in the component vault
            // Manifest -> 03_issuer_mint_agent_badge.rtm
            // Alternative Manifest -> 03a_issuer_mint_agent_badge_manifest.rtm -> used to demonstrate minting without a method call
        pub fn issuer_mint_agent_badge(&mut self, issuer_badge: NonFungibleProof, company_lei: String,local_id: u64,) -> NonFungibleBucket {
            let checked_proof = issuer_badge.check_with_message(
                self.issuer_badge_manager.address(),
                "Invalid Issuer Badge as Proof!",
            );

            let issuer_data = checked_proof.non_fungible::<IssuerBadge>();
            let issuer_badge_id = issuer_data.resource_address();
            let issuer_badge_local_id = issuer_data.local_id();
            let issuer_agent_badge: NonFungibleBucket = self
                .system_badge_vault
                .authorize_with_amount(1, || {
                    self.issuer_agent_badge_manager.mint_non_fungible(
                        &NonFungibleLocalId::Integer(local_id.into()),
                        IssuerAgentBadge {
                            // links the issuer agent to the issuer
                            // purpose is to ensure an issuer agent is interacting only with instruments of that particukar issuer
                            issuer_badge_id: issuer_badge_id.clone(),
                            issuer_badge_local_id: issuer_badge_local_id.clone(),
                            company_lei: company_lei,
                        },
                    )
                })
                .as_non_fungible();
            issuer_agent_badge
        }

            // Issuer defines the security static data which is set at the instrument metadata level
            // Issuer Agent is responsible for defining the instrument lifecycle events as seen in the minting rule.
            // Issuer Badge Global Id is set on the instrument metadata level to support multi issuers on the platform.
            // Generally veers towards system_badge approach as fine grained permissioning using methods will be required.
            // Metadata covers instrument metadata provided by the issuer but also adds metadata fields typically provided by a data vendor
            // Data Vendor fields uses example for SFTR (Security Finance Transaction Reporting fields used for Regulatory purposes)
            // Majority of Metadata fields hardcoded for now.
        pub fn issuer_create_instrument(&mut self, issuer_badge: NonFungibleProof, security_type: String, security_form: String, name: String, symbol: String) {
            assert!(
                security_type == "Equity" || security_type == "Bond",
                "Invalid Security Type"
            );
            assert!(
                security_form == "Bearer" || security_form == "Registered",
                "Invalid Security Form"
            );

            let checked_proof = issuer_badge.check_with_message(
                self.issuer_badge_manager.address(),
                "Invalid Issuer Badge as Proof!",
            );
            // identify the issuer from the issuer badge proof
            let issuer_data = checked_proof.non_fungible::<IssuerBadge>();
            let issuer_id = issuer_data.resource_address();
            let issuer_local_id = issuer_data.local_id();
            // combine badge address and local id to determine the issuer's global id
            let issuer_global_id: NonFungibleGlobalId =
                NonFungibleGlobalId::new(issuer_id.clone(), issuer_local_id.clone());

            let instrument_manager =
                ResourceBuilder::new_integer_non_fungible::<InstrumentLifecycleData>(
                    OwnerRole::Fixed(rule!(require(self.owner_badge))), //set the owner as owner for now
                )
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(self.system_badge));
                        metadata_setter_updater => OWNER;
                        metadata_locker => rule!(require(self.system_badge));
                        metadata_locker_updater => OWNER;
                    },
                    init {
                        "name" => name, locked;
                        "symbol" => symbol, locked;
                        //"issuer_local_id" => issuer_local_id, locked;
                        "instrument_status" => "verified", updatable;
                        // add the issuer identifier to the instrument metadata
                        "issuer_global_id" => issuer_global_id.clone(), locked;
                        "security_type" => security_type, locked;
                        "security_form" => security_form, locked;
                        "subscription_status" => "open", updatable;
                        "subscription_amount" => dec!(1000000), updatable;
                        "subscription_price" => dec!(100.00), updatable;
                        "issuance_amount" => dec!(0), updatable;
                        "issuance_price" => dec!(100.00), updatable;
                        "currency" => XRD, updatable;
                        "coupon" => "5%", updatable;
                        // adds sftr codes typically provided by a data vendor or third party
                        // sftr_security_type = {
                        //     "GOVS": "Government securities",
                        //     "SUNS": "Supra-nationals and agencies securities",
                        //     "FIDE": "Debt securities (including covered bonds) issued by banks and other financial institutions",
                        //     "NFID": "Corporate debt securities (including covered bonds) issued by non-financial institutions",
                        //     "SEPR": "Securitized products (including CDO, CMBS, ABCP)",
                        //     "MEQU": "Main index equities (including convertible bonds)",
                        //     "OEQU": "Other equities (including convertible bonds)",
                        //     "OTHR": "Other assets (including shares in mutual funds)"
                        // }
                        "sftr_issuer_lei" => "LEI000000000000", updatable;
                        "sftr_issuer_juristiction" => "US", updatable;
                        "sftr_security_type" => "GOVT", updatable;
                        "sftr_security_quality" => "INVG", updatable;
                        "sftr_security_rating" => "AAA+", updatable;
                    }
                ))
                .mint_roles(mint_roles! {
                    minter => rule!(
                        require(self.issuer_agent_badge_manager.address()) ||
                        require(self.system_badge)
                    );
                    minter_updater => OWNER;
                })
                .create_with_no_initial_supply();

            // add the instrument to the instrument_manager
            self.instrument_manager.push(instrument_manager);
            // add the resource address, initializing the instrument version with 0
            self.instrument_version.insert(instrument_manager.address(), 0u64);
        }

            // returns the instruments currently set up
        pub fn get_instruments(&self) -> Vec<ResourceManager> {
            self.instrument_manager.iter().cloned().collect()
        }

            // method restricted to the issuer for metadata fields issuer is responsible for
        pub fn issuer_update_instrument_metadata(&mut self, issuer_badge: NonFungibleProof, instrument: ResourceAddress,
            key: String, value: String) {

            // restricts the issuer from updating the sftr_ fields
            assert!(
                !key.starts_with("sftr"),
                "Issuer not permissioned to update SFTR metadata fields"
            );

            let checked_proof = issuer_badge.check_with_message(
                self.issuer_badge_manager.address(),
                "Invalid Issuer Badge supplied as Proof!",
            );

            let issuer_data = checked_proof.non_fungible::<IssuerBadge>();
            // derive the issuer global id from the badge proof
            let issuer_id = issuer_data.resource_address();
            let issuer_local_id = issuer_data.local_id();
            let issuer_global_id: NonFungibleGlobalId =
                NonFungibleGlobalId::new(issuer_id.clone(), issuer_local_id.clone());

            // check the instrument already exists in the instrument_manager
            if !self
                .instrument_manager
                .iter()
                .any(|manager| manager.address() == instrument)
            {
                panic!("Instrument Resource Address not found");
            }

            let instrument_manager = ResourceManager::from_address(instrument);
                // retrieves the issuer global id set on the instrument level
            let bond_issuer_global_id: NonFungibleGlobalId = instrument_manager
                .get_metadata("issuer_global_id")
                .unwrap()
                .expect("issuer global id field not set on the instrument metadata");

                // compares the issuer global id from the badge with the id set on the instrument
                // ensures the issuer is updating only instruments issued by that issuer
            assert_eq!(
                bond_issuer_global_id, issuer_global_id,
                "Issuer is not owner of this instrument"
            );
                // authorizes the metadata update using the system badge stored in a component vault
            self.system_badge_vault.authorize_with_amount(1, || {
                instrument_manager.set_metadata(key, value);
            });
        }

            // sets the subscription status to "open" having checked the instrument set up has been finalized by the issuer
            // requires further updating to ensure the issuer "owns" this security as per previous method above
        pub fn issuer_open_subscription(&mut self, instrument: ResourceAddress) {

            let instrument_manager = ResourceManager::from_address(instrument);

            assert_eq!(
                self.instrument_manager.contains(&instrument_manager),
                true,
                "The instrument does not exist in the instrument manager"
            );

            let instrument_status: Option<String> = instrument_manager.get_metadata("instrument_status").unwrap();

            assert_eq!(
                instrument_status.unwrap(), "verified", "The issuer has not verified the instrument"
            );

            self.system_badge_vault.authorize_with_amount(1, || {
                instrument_manager.set_metadata("subscription_status", "open".to_string());
            });
        }

            // issuer closes the subscription and sets the issuance amount on the instrument based on the subscription amount counter
        pub fn issuer_close_subscription(&mut self, instrument: ResourceAddress) {
            let instrument_manager = ResourceManager::from_address(instrument);

            assert_eq!(
                self.instrument_manager.contains(&instrument_manager),
                true,
                "The instrument does not exist in the instrument manager"
            );

            let issuance_amount = self.subscribed_amount;
            self.system_badge_vault.authorize_with_amount(1, || {
                instrument_manager.set_metadata("subscription_status", "closed".to_string());
                instrument_manager.set_metadata("issuance_amount", issuance_amount);
                // resets the counter to 0 to allow the next subscription to proceed, quickfix for now
            self.subscribed_amount = 0.into();
            });
        }
            // method to allow the issuer agent to add lifecycle events to an issuer's instrument e.g. coupon payments
            // simplified set up where a percent is passed in to represent fixed coupon %, for issuance this would be 100%
            // manifest -> 13_agent_add_instrument_lifecycle_issuance.rtm -> typically add the issuance initially
            // manifest -> 17_agent_add_instrument_lifecycle_coupons.rtm -> typically added during life of the security as required
        pub fn agent_add_instrument_lifecycle(&mut self, instrument: ResourceAddress, action_type: String,
            percent: Decimal) {

            let instrument_manager = ResourceManager::from_address(instrument);

            assert_eq!(
                self.instrument_manager.contains(&instrument_manager),
                true,
                "The instrument does not exist in the instrument manager. Contact the Issuer!"
            );

            let instrument_status: Option<String> = instrument_manager.get_metadata("instrument_status").unwrap();

            assert_eq!(
                    instrument_status.unwrap(), "verified", "The issuer has not verified the instrument"
            );
                // restricted to certain corporate actions for initial design
            assert!(["Issuance", "Coupon", "Dividend"].contains(&action_type.as_str()), "Invalid action type");

                // increments the version by 1, which was initially 0 in the initialization step above
            let mut instrument_version: u64 = *self.instrument_version.get(&instrument).unwrap();
                instrument_version += 1;
                info!("instrument version: {:?}", instrument_version);

                // force the first lifecycle action to be of type Issuance to simplify set up for now
            if instrument_version == 1 {
                assert_eq!(action_type, "Issuance", "The first instrument lifecycle nft must be an issuance");
            }

            let instrument_bucket: NonFungibleBucket = self
                .system_badge_vault
                .authorize_with_amount(1, || {
                    instrument_manager.mint_non_fungible(
                        &NonFungibleLocalId::Integer(instrument_version.into()),
                        InstrumentLifecycleData {
                            action_type: action_type.to_string(),
                            percent: percent,
                            available: true,
                        },
                    )
                })
                .as_non_fungible();

            info!("instrument address: {:?}",instrument_bucket.resource_address());

                // determine the global_id of this lifecycle action and derive the global_id of the next lifecycle action
                // recorded in the instrument_lifecycle structure
                // used later to determine the sequence of corporate actions to be performed when investor claims the corporate action
            let resource_address = instrument_bucket.resource_address();
            let local_id = NonFungibleLocalId::integer(instrument_version);
            let global_id: NonFungibleGlobalId =
                NonFungibleGlobalId::new(resource_address, local_id);

                // record the current instrument version per instrument
            self.instrument_version
                .insert(instrument_manager.address(), instrument_version);
            let next_local_id = NonFungibleLocalId::integer(instrument_version + 1);
            let next_global_id = NonFungibleGlobalId::new(resource_address, next_local_id);
                // record the global id related to the lifecycle action and the global id related to the next lifecycle action
                // to be reviewed if more robust approach to achieve this
            self.instrument_lifecycle.insert(global_id, next_global_id);

                // insert the nft into the instrument vault, should always match on Some
            let instrument_vault = self.instrument_vault.get_mut(&instrument);
            match instrument_vault {
                Some(vault) => {
                    vault.put(instrument_bucket);
                }
                None => {
                    let instrument_vault = NonFungibleVault::with_bucket(instrument_bucket);
                    self.instrument_vault.insert(instrument, instrument_vault);
                }
            };
        }

            // issuer agent issues the fungible securities based on the lifecycle actions set on the instrument
            // these are the securities the investor will receive post the subscription process
            // initally set up to create a new fungible security for each lifecycle action present on the instrument
            // can be rerun to issue further new security versions as more instrument lifecycle events are added to the instrument
            // at each corporate action investor returns the security version they are holding, receives the coupon and receives a new security version
            // manifest -> 14_agent_issue_lifecycle_securities.rtm -> typically run when "Issuance" lifecycle added to mint the initial security version
            // manifest -> 18_agent_issue_lifecycle_securities.rtm -> typically run when "Coupon / Dividends" lifecycles added to mint the next security version(s)
            // note these manifest are the same as the method just mints any outstanding security versions
        pub fn agent_issue_lifecycle_securities(&mut self, instrument: ResourceAddress) {

            let instrument_manager = ResourceManager::from_address(instrument);

            assert_eq!(
                self.instrument_manager.contains(&instrument_manager),
                true,
                "The instrument does not exist in the instrument manager"
            );

            let subscription_status: Option<String> = instrument_manager
                .get_metadata("subscription_status")
                .unwrap();

                // ensures the subscription process is closed before minting, though this should be caught when minting the instrument lifecycle NFT'S
            assert_eq!(
                subscription_status.unwrap(),
                "closed",
                "Subscription is still open"
            );
                // collects all instrument lifecycles (global id's) across all instrument's initially
            let securities_to_issue: Vec<_> = self.instrument_lifecycle.keys().cloned().collect();

                // checks if the fungible security related to the instrument lifecycle global id already exists
            for security in securities_to_issue {
                if self.security_holdings_vault.contains_key(&security) {
                    continue;
                }
                // checks if the instrument related to the instrument lifecycle global id exists
                // precautionary check
                let (resource_address, local_id) =
                    NonFungibleGlobalId::into_parts(security.clone());
                if resource_address != instrument_manager.address() {
                    continue;
                }
                // checks if the instrument lifecycle nft related to the global id exists
                // precautionary check
                assert_eq!(
                    instrument_manager.non_fungible_exists(&local_id),
                    true,
                    "the lifecycle does not exist yet"
                );
                info!("adding lifecycle for following security {:?}", security);

                // retrieve symbol & name of instrument to also assign to the fungible security
                let symbol: String = instrument_manager
                    .get_metadata("symbol")
                    .unwrap()
                    .expect("Symbol field not set on the instrument metadata");
                let name: String = instrument_manager
                    .get_metadata("name")
                    .unwrap()
                    .expect("Name field not set on the instrument metadata");
                //retrieve the issuance ammount set on the instrument at the close of the subscription process
                let issuance_amount: Option<Decimal> = instrument_manager
                    .get_metadata("issuance_amount")
                    .unwrap_or(None);
                // retrieve sftr codes, let's take just one ot two for now
                let sftr_security_type: String = instrument_manager
                .get_metadata("sftr_security_type")
                .unwrap()
                .expect("sftr_security_type field not set on the instrument metadata");
                let sftr_security_rating: String = instrument_manager
                .get_metadata("sftr_security_rating")
                .unwrap()
                .expect("sftr_security_rating field not set on the instrument metadata");
                    // set this amount as the supply of the fungible security to be issued
                let supply = issuance_amount.unwrap();
                    // look up the next lifecycle global id based on the current global id
                let next_global_id = self.instrument_lifecycle.get(&security).unwrap();

                let security_bucket: FungibleBucket = ResourceBuilder::new_fungible(
                    OwnerRole::Fixed(rule!(require(self.owner_badge))), //set the owner as owner for now
                )
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(
                            require(self.issuer_agent_badge_manager.address())); //issuer agent can update metadata on non locked fields
                        metadata_setter_updater => OWNER;
                        metadata_locker => rule!(
                            require(self.issuer_agent_badge_manager.address()));
                        metadata_locker_updater => OWNER;
                    },
                    init {
                        "symbol" => symbol, locked;
                        // add combination of instrument name and use local_id for now to signify the fungible security version being minted
                        "name" => format!("{}-{}", name, local_id), locked;
                        // add instrument lifecycle id's to fungible security metadata for cross referencing
                        "instrument_global_id" => security.clone(), locked;
                        "instrument_resource_address" => resource_address.clone(), locked;
                        "instrument_local_id" => local_id.clone(), locked;
                        "instrument_next_global_id" => next_global_id.clone(), locked;
                        "sftr_security_type"  => sftr_security_type, updatable;
                        "sftr_security_rating"  => sftr_security_rating, updatable;
                    }
                ))
                .mint_roles(mint_roles! {
                    minter => rule!(
                        require(self.issuer_agent_badge_manager.address()) || // allows issuer agent to mint directly in a manifest
                        require(self.system_badge) // or using this method which is only permissioned for the issuer agent
                    );
                    minter_updater => OWNER;
                })
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all); // to be updated later to restict who can burn the securities, only prior version securities to be burned from component vault when returned by the investor
                    burner_updater => OWNER;
                })
                .divisibility(DIVISIBILITY_MAXIMUM)
                .mint_initial_supply(supply);

                    // look up the fungible security vault which maps to the lifecycle global id key
                    // expected to match on None as method is creating the securities
                let vault = self.security_holdings_vault.get_mut(&security);
                match vault {
                    Some(security_vault) => {
                        security_vault.put(security_bucket);
                    }
                    None => {
                        let security_vault = FungibleVault::with_bucket(security_bucket);
                        self.security_holdings_manager.insert(
                            security_vault.resource_address(),
                            instrument_manager.address(),
                        );
                        self.security_holdings_vault
                            .insert(security.clone(), security_vault);
                    }
                };
                    // once the fungible security has been created the lifecycle entry is then removed
                self.instrument_lifecycle.remove(&security);
                info!("security_holdings_vault now contains {:?}", self.security_holdings_vault);
            }
        }

            // initially supporting Bearer Security where the investor or holder of the security claims the lifecycle or corporate action from the issuer
            // investor returns the current version of the fungible security
            // based on the metadata of this security the next instrument lifecycle to be be processed can be detemriend if any
            // investor receives the coupon & the next security version in return
        pub fn investor_claim_corporate_action(&mut self, security_holding: FungibleBucket) -> (FungibleBucket, FungibleBucket) {

            let security_holding_address = security_holding.resource_address();

                //check the fungible security provided relates to this security manager component
            assert_eq!(
                self.security_holdings_manager
                    .contains_key(&security_holding_address),
                true,
                "The security does not exist in the security manager"
            );
                // determine the next lifecycle or corporate action to be processed if any
            let security_holding_manager = security_holding.resource_manager();
            let global_id: NonFungibleGlobalId = security_holding_manager
                .get_metadata("instrument_next_global_id")
                .unwrap()
                .expect("The next security version is unknown");
            info!("Processing next lifecycle action with Global Id: {:?} !", &global_id);

            let (resource_address, local_id) = NonFungibleGlobalId::into_parts(global_id.clone());

            let instrument_manager = ResourceManager::from_address(resource_address);
                // checks if lifecycle action exists in the instrument data as specified on the fungible security
            assert!(instrument_manager
                .non_fungible_exists(&local_id),
                "There are no currently no lifecycle events available to be processed for this Global Id");

            let lifecycle_data: InstrumentLifecycleData =
                instrument_manager.get_non_fungible_data(&local_id);
            let action_type = lifecycle_data.action_type;

                // currently only supports coupon payments
            assert_eq!(
                action_type, "Coupon",
                "No coupon payment due for this instrument"
            );

            info!("security_holdings_vault now contains {:?}", self.security_holdings_vault);
            assert!(
                    self.security_holdings_vault.contains_key(&global_id),
                    "The next version security to be issued is not available in the security holding vault"
                );
                // process the coupon payment first (and later the next version of the securities)
                // determine the coupon amount based on amount of securities passed in the bucket and coupon percent set on the instrument nft
            let percent = lifecycle_data.percent;
            info!("Coupon % is: {} !", percent);
            let amount = security_holding.amount();
            info!("Investor Security Holding is: {} !", amount);
            let coupon_payment = amount * percent / 100;
            info!("Coupon Payment due is: {} !", coupon_payment);

            // let currency: ResourceAddress = security_holding_manager.get_metadata("currency").unwrap().expect("Currency field not set on the security metadata");
            // assert_eq!(
            //     currency, XRD,
            //     "Coupon payment currency is not XRD"
            // );
                // no check performed if coupon currency is set to XRD for now
                // checks if enough cash in the XRD vault to pay out the coupon
            info!("XRD cash balance is currently: {} !", self.cash_holding_vault.amount());
            assert!(
                self.cash_holding_vault.amount() >= coupon_payment,
                "Insufficient funds to pay lifecycle event"
            );
            let coupon_bucket: FungibleBucket = self.cash_holding_vault.take(coupon_payment);

            // now issue the next version securities by retrieving the vault of previously minted fungible securities
            // should always match Some
            let vault = self.security_holdings_vault.get_mut(&global_id);
            info!("vault {:?}", vault);
            let mut security_bucket: Option<FungibleBucket> = None;

            match vault {
                Some(security_vault) => {
                    security_bucket = Some(security_vault.take(amount));
                }
                None => {
                    info!("Error");
                }
            }
            // burn the previous version of the securities passed in by the investor
            security_holding.burn();
            (coupon_bucket, security_bucket.unwrap())
        }

            // simplified check that is available to anyone to see if they are a qualified investor
            // returns an investor badge which is required for subscribing to a security
        pub fn investor_check_kyc(&mut self, country: String, favourite_int: u64) -> Bucket {
            assert!(
                country != "US" && country != "USA",
                "US investors are not allowed to mint Investor Badges"
            );
            assert_ne!(
                favourite_int, 666,
                "666 investors are not allowed to mint Investor Badges"
            );
            info!("Investor Badge minted! Subscriptions are now open!");

            let investor_badge_bucket: Bucket = self.investor_badge_manager.mint_non_fungible(
                &NonFungibleLocalId::Integer(favourite_int.into()),
                InvestorBadge { country: country },
            );
            investor_badge_bucket
        }

            // investors can subscribe to security using this protected method specifying the instrument and amount to be subscribed
            // in return receives a subscription NFT which outlines the details to satisfy the escrow process
            // rejected if the issuance is already over-subscribed
        pub fn investor_subscribe(&mut self,instrument: ResourceAddress,subscribe_amount: Decimal,
        ) -> NonFungibleBucket {

            let instrument_manager = ResourceManager::from_address(instrument);
                // Issuer is required to have set the instrument up initially
            assert!(
                self.instrument_manager.contains(&instrument_manager),
                "The instrument does not exist in the instrument manager. Check the resource address!"
            );

            let subscription_status: Option<String> = instrument_manager
                .get_metadata("subscription_status")
                .unwrap();

            assert_eq!(
                subscription_status.unwrap(),
                "open",
                "Subscription is not yet open"
            );

            let current_subscribed_amount = self.subscribed_amount + subscribe_amount;
            let subscription_total_amount: Option<Decimal> = instrument_manager
                .get_metadata("subscription_amount")
                .unwrap();

            assert!(
                current_subscribed_amount <= subscription_total_amount.unwrap(),
                "Requested subscribed amount exceeds remaining available amount"
            );

            self.subscribed_amount = current_subscribed_amount;
            let subscription_price: Option<Decimal> = instrument_manager
                .get_metadata("subscription_price")
                .unwrap();
                // requires updating for safe overflow handling
            let payment_amount = subscribe_amount / 100 * subscription_price.unwrap();
            let symbol: String = instrument_manager
                .get_metadata("symbol")
                .unwrap()
                .expect("Symbol field not set on the instrument metadata");

            let subscription_bucket: NonFungibleBucket = self
                .subscription_manager
                .mint_non_fungible(
                    &NonFungibleLocalId::Integer(1.into()),
                    SubscriptionEscrowTerms {
                        // playing around here
                        party: "investor".to_string(),
                        symbol: symbol,
                        // sets the escrow details here based on resource investor is receiving and the payment details
                        // sets the status to "pending"
                        rec_resource: instrument_manager.address(),
                        rec_qty: subscribe_amount,
                        pay_resource: self.cash_holding_vault.resource_address(),
                        pay_amount: payment_amount,
                        escrow_status: "pending".to_string(),
                    },
                )
                .as_non_fungible();

            subscription_bucket
        }

            // investor is required to transfer the payment for the securities subscribed
            // provide the subscription nft as proof and the payment token
        pub fn investor_transfer_payment(&mut self, subscription_proof: NonFungibleProof, mut payment_token: FungibleBucket,
        ) -> FungibleBucket {

            let checked_proof = subscription_proof
                .check_with_message(self.subscription_manager.address(), "Incorrect proof!");

            let subscription = checked_proof.non_fungible::<SubscriptionEscrowTerms>();
            let subscription_data = subscription.data();
            let local_id = subscription.local_id();
            let escrow_status = subscription_data.escrow_status;
                // simplified check to see if payment was already made
            assert_eq!(escrow_status, "pending", "The escrow is not in pending status");
                // determine the amount actually being paid matches what is expected to be paid based on subscription proof data
            let pay_resource_due = subscription_data.pay_resource;

            assert_eq!(
                payment_token.resource_address(),
                pay_resource_due,
                "The payment resource received does not match the payment resource due"
            );

            let pay_amount_due = subscription_data.pay_amount;
            info!("pay_amount_due {:?}", pay_amount_due);
            info!("pay_provided {:?}", payment_token.amount());


            assert!(
                payment_token.amount() >= pay_amount_due,
                "The payment amount received is less than the payment amount due"
            );

                // updates the status to settled if all checks are successful
            self.system_badge_vault.authorize_with_amount(1, || {
                self.subscription_manager.update_non_fungible_data(
                    &local_id,
                    "escrow_status",
                    "settled".to_string(),
                )
            });
                // inserts payment into vault and returns any overpayment
            let payment_received = payment_token.take(pay_amount_due);
            self.cash_holding_vault.put(payment_received.into());
            payment_token
        }

            // investor can cancel the payment already made
            // small deviation from a typical escrow here -> can only cancel payment up to the point subscription closes
            // can be relaxed to ensure investor can cancel payment if issuer does not provide the securities
            // in this case we pass in the badge (tather than using the proof approach) to play around
            // investor receives the badge back and the refunded payment amount
        pub fn investor_cancel_payment(&mut self, subscription_badge: NonFungibleBucket,) -> (NonFungibleBucket, Bucket) {

            assert_eq!(
                self.subscription_manager.address(),
                subscription_badge.resource_address(),
                "Invalid subscription badge provided"
            );

            let local_id = subscription_badge.non_fungible_local_id();
            let subscription_data: SubscriptionEscrowTerms =
                self.subscription_manager.get_non_fungible_data(&local_id);
            let escrow_status = subscription_data.escrow_status;
            let instrument_id = subscription_data.rec_resource;
            let instrument_manager = ResourceManager::from_address(instrument_id);
            info!("current escrow status: {:?}", escrow_status);

            assert_eq!(escrow_status, "settled", "The esrow status is not settled");

            let subscription_status: Option<String> = instrument_manager
                .get_metadata("subscription_status")
                .unwrap();
            info!("current subscription status: {:?}", subscription_status);

            assert_ne!(
                subscription_status.unwrap(),
                "closed",
                "The subscription is now closed, no refunds allowed"
            );

            let pay_amount_due = subscription_data.pay_amount;
                // reset the status from "settled" to "pending"
            self.system_badge_vault.authorize_with_amount(1, || {
                self.subscription_manager.update_non_fungible_data(
                    &local_id,
                    "escrow_status",
                    "pending".to_string(),
                )
            });
            info!("escrow status is now: {:?}", escrow_status);

            let refund_payment: Bucket = self.cash_holding_vault.take(pay_amount_due).into();
                // return the badge and the payment
                // investor can reconsider and transfer the payment again if desired
            (subscription_badge, refund_payment)
        }

            // at this point, the subscription is expected to be closed and the issuer has minted the fungible securities for the investor to claim
            // check the subscription badge if the payment has settled
            // return the fungible securities to the investor
            // insert the subscription nft into the component vault
            // later, the issuer can then withdraw the issuance proceeds based on the subscription badges returned to the vault

        pub fn investor_claim_security(&mut self, subscription_badge: NonFungibleBucket,
        ) -> FungibleBucket {

            assert_eq!(
                self.subscription_manager.address(),
                subscription_badge.resource_address(),
                "Invalid subscription badge provided"
            );

            let local_id = subscription_badge.non_fungible_local_id();
            let subscription_data: SubscriptionEscrowTerms =
                self.subscription_manager.get_non_fungible_data(&local_id);
            let escrow_status = subscription_data.escrow_status;

            assert_eq!(
                escrow_status, "settled",
                "The escrow status is not settled"
            );

            let instrument = subscription_data.rec_resource;
            let non_fungible_local_id = NonFungibleLocalId::Integer(1.into());
            let security_qty = subscription_data.rec_qty;
            let global_id: NonFungibleGlobalId =
                NonFungibleGlobalId::new(instrument, non_fungible_local_id);
            let vault = self.security_holdings_vault.get_mut(&global_id);
            info!("vault {:?}", vault);
            let mut security_bucket: Option<FungibleBucket> = None;

            match vault {
                Some(security_vault) => {
                    security_bucket = Some(security_vault.take(security_qty));
                }
                None => {
                    info!("No supply");
                }
            }
            self.subscription_manager_vault.put(subscription_badge.into());
            security_bucket.unwrap()
        }

            // issuer specifies the instrument that went through subscription phase to receive proceeds to payments collected

        pub fn issuer_claim_cash(&mut self, instrument: ResourceAddress) -> FungibleBucket {

                // retrieve up to 100 nft subscriptions returned to the vault
                // need to figure out hwo to ensure all are retrieved, hardcoded for nwo
            let ids = self.subscription_manager_vault.non_fungible_local_ids(100);
            let mut issuer_amount_due = Decimal::zero(); 


                // iterate through all subscription nft's in the vault
                // determine those nft's that are related to this particular instrument, ignoring other nft's related to other issuances
                // can be reworked, for now all subscriptions across all instruments are stored together
            for id in ids.iter() {
                info!("processing id: {:?}", id);
                let subscription_badge = self.subscription_manager_vault.take_non_fungible(&id);
                let local_id = subscription_badge.non_fungible_local_id();
                let subscription_data: SubscriptionEscrowTerms =
                    self.subscription_manager.get_non_fungible_data(&local_id);

                let rec_resource = subscription_data.rec_resource;
                info!("rec_resource id: {:?}", rec_resource);
                    // check this nft relates to the instrument the issuer specified
                if rec_resource != instrument {
                    continue;
                }
                    // for subscription nfts that match the instrument requested, accumulate the payment amounts provided by the investors
                let pay_resource = subscription_data.pay_resource;
                let pay_amount = subscription_data.pay_amount;
                info!("pay_amount: {:?}", pay_amount);
                issuer_amount_due = issuer_amount_due + pay_amount; 
                info!("issuer_amount_due: {:?}", issuer_amount_due);

                assert!(
                issuer_amount_due > Decimal::zero(), "No funds to withdraw"
                );
                assert_eq!(pay_resource, self.cash_holding_vault.resource_address(), "The payment resource received does not match the payment resource due"); 
                // these matching subscription are then instructed to be burnt from the vault
                subscription_badge.burn();
            }
                info!("issuer_amount_new_due: {:?}", issuer_amount_due);
                    // issuer receives one lump sum for this particular subscription
                self.cash_holding_vault.take(issuer_amount_due)
        }

            // as issuer may have removed all issuance proceeds, issuer can deposit funds to ensure coupons can be paid out
        pub fn issuer_deposit_funds(&mut self, cash_token: FungibleBucket) {
            assert_eq!(cash_token.resource_address(), self.cash_holding_vault.resource_address(), "Invalid token address");
            self.cash_holding_vault.put(cash_token);
        }
    }
}
