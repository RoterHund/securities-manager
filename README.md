## Overview
This is the submission to complete the Scrypto 101 course

The software extends the Escrow challenge with the primary focus to explore the
possibilities to model financial instruments, in particular securities i.e. Bonds
& Equities using Scrypto. I would like to be in a position to publish a series
and develop a prototype "Showcasing Radix for Financial Institutions" based on my
own background of designing trading systems for banks, corporates and clearinghouses.
The first step requires validating if and how scrypto can model a security as
this is a fundamental component to enable interesting solutions to be built

Securities take 2 forms, Bearer Securities and Registered Securities. The initial
focus is on modelling Bearer Securities. The goal is to find a design approach to
allow the securities to be fungible, allowing an investor to sell or lend the
securities and at the same time ensuring the investor can receive the rights they
are entitled to over the lifetime of the security. The focus is primarily on
modelling a fixed rate bond and allowing the bond holder to receive the issuance
and the coupons as the initial prooof of concept.

The design is implemented from the perspective of a platform or technology
provider, the "Security Manager Owner". The Owner maintains the platform and
enables issuers, issuer agents and investors to interact as follows:

- issuer is responsible for defining the security characteritics, performing kyc
checks on the investors, opening and closing the subscription process
- the issuer appoints an agent who is responsible for the lifecycle of the
security, defining the coupons to be paid and issuing the securities to the investor
- the investor subscribes for a quantity of securities on offer, subject to passing the issuer KYC checks. The investor receives the securities on issuance after the subscription period closes and makes claims for coupons over the lifetime od the security

## Bearer Securities Overview
Historically the holder of a Bearer Security in paper format (Certificate) presented the paper to the issuer as evidence of ownership. The certificate represented a quantity of securities and coupons were attached to the certificate.

Investors cut off the coupon at payment date and in return received the coupon from the issuer. These certificates were fungible and could be passed around from investor to investor, whoever presented it received the rights.

In summary, these were similar to holding cash but with rights attached.
Later custodians provided services to investors to safekeep these certificates, claimed the coupons and passed them on to the investor.

This solution ignores the role of the custodian as this is a safekeeping role and focuses on the instrinsic properties of bearer securities.

## Bearer Securities Solution Overview
The solution seeks to explore how to issue these securities as fungible securities, allow the investor to hold the securities and claim the coupons.

Any solution that uses NFT's to respresent these assets seems sub-optimal for mass issuance of securities on a DLT. Securities are not issued to live in isolation, they are often grouped together by a risk profile for purposes and exchanged at a risk profile level e.g. by security type, rating category, issuer and exchanged at this level. Issuing securities as NFT's seems to add additional complexity and friction in particular in the absence of all participant's following an agreed NFT standard on modelling them. While issuing these as fungible tokens does not magically negate the need for standards, it would seem to be a better starting point, for the reason alone these are intrinsically fungible in nature.

 - Issuer sets up an instrument as an NFT which describes the terms of the issuance
   -  The metadata of the NFT represents the reference data of the security
   -  The individual NFT's represent the rights pertaining to the security
      e.g issuance, series of coupons uniquely identifiying using the global_id
   -  These NFT's are stored in a component vault and not withdrawable
   -  This could be considered an immobilized global bearer bond certificate


 - Investors subscribe to the security for a quantity of securities, and an issuance
   amount is determined at the close of the subscription processs.
   -  The metadata of the bond is updated with this amount

 - Issuer Agent mints fungible securities that reference the instrument NFT
   - Investor claims this securities, holding these securities are the proof of ownership
   - A new version of the fungible securities is created for each NFT (at global_id)
   - The investor transfers the fungible securities on each coupon date as evidence
     of being the owner of the securities to the component
   - In return the investor receives a coupon payment and a new version of the securities
   - The securities returned to the issuer are burned

 - The versioning concept is introduced to ensure the coupons or life cycle events are
   not paid out more than once.
   -  Multiple versions of the security can be in existence in parallel, however each
      version can be linked back to the NFT instrument
   -  However investors should be motivated to claim the coupons as they are paid out



As the securities are now in fungible form, the investor can sell on some or all of the securities, lend the securities, use them as collateral etc. However securities that are lent or posted as collateral typically are covered by securities finance legal agreements (GMSLA for Securities Lending, GMRA for Repo transactions). These agreements cover what happens to the rights embedded in the securities when in possession of the borrower over the loan period or collateral holding period.

The owner who has transferred title "temporarily" is entitled to the coupon payments. Typically in collateral management processes, these securities are returned to the lender over this period, often are ineligible to be posted as collateral in the period. However this is primarily to avoid the complexity, operational burden in making a claim against the borrower or collateral receiver to pass on the coupons to the owner that they have received while in possession of the securities.

At the same time, there are securities which are actively borrowed for the purpose of exercising the rights e.g. rquity voting rights at AGM's. This is to be considered later.


## Bearer Securities Solution Extension
This solution stops at the point where the securities are issued and the holder claims the coupon payments. The intention is to explore the solution further as follows:

- investor receives the securities and deposits them into collateral pools managed by the security manager and in return receives a collateral token
- the collateral tokens will be grouped into certain risk profiles depending on attributes of the individual securities e.g. GOVT A Rated Collateral Token, MBS B Rated Collateral Token etc.
- the setting of the particular metadata fields to be determine the risk profile should be outside the control of the "owner" of the security and performed by a third party
- In triparty business, these is similar to the concept of counterparties signing schedules, where the schedule describes the acceptable collateral in a lending transaction e.g. Bond Rated BBB+ of higher, Security Types in GOVS, MBS, Issuer Country not in List of Sanctioned Countries, Equities traded on Main Exchanges. Concentration Rules are also typically applied e.g. no more than 40% Equities by Market Value. Similar concept applies to bilateral collateral management processes where acceptable collateral is listed in the legal agreement
- This token than can be lent out, posted as collateral or used to borrow another security or cash, swapped in collateral upgrade or downgrade trades etc.
- Investor as owner of the securities will then be enabled to retrieve the coupon as follows:
   - Remove the securities from the collateral pool
   - Transfer the securities to the issuer, receive the coupon payment
   - Receives a new version of the securities to be deposited back into the collateral pool
   - This is required to be performed in one transaction to ensure the collateral pool is
     always backed by securities, working similar to the mechanics of a flash loan
- publishing of events that can be retrieved and messages derived to be importing in bank trading
   system
- addition of royalties for the platform owner when issuer interacts with the system


## Solution Design Comments
The solution mixes and matches verious design patterns. The intention is not to assert any particular design choice is the best choice for each piece of functionality. Rather the goal was to learn how the different approaches can be implemented.

The implementations are not intended to be robust, rather the primary focus is more a proof of concept of scrypto possibilities to model real financial instrument use cases.

This solution would be intended to be split into multiple component, for now it remains as one.

The design of the escrow functionality was a secondary consideration, it is hoped it is rich enough to suffice to pass the Scrypto 101 course.

## Burning Question
Is there another approach to implement this that does not require a versioning of the securities?
If so, does it require using NFT's to represent the security holdings?
What's the Radix guidelines for implementing RWA where rights are involved?

## Using the Security Manager

### Setup

1. First, clone the repository if you have not done so, and then change
   directory to this example.

   ```
   git clone https://github.com/RoterHund/securities-manager.git

   cd securities-manager
   ```

2. Run the setup script.

   On Linux or MacOS:

   ```sh
   source ./setup.sh
   ```

   This publishes the security manager using resim and creates 4 accounts
   - owner account
   - issuer account
   - issuer agent account
   - investor account

### Security Manager Instantiation
1. Instantiate the component by using the `01_instantiate_securities_manager.rtm`
   manifest:

   ```sh
   resim run manifests/01_instantiate_securities_manager.rtm
   ```
   This creates the following resources
   - Securities Manager Owner Badge (OWNER") -> deposited to owner account
   - Platform Badge ("AUTO") -> deposited to component vault
   - Issuer Badge ("ISSUER") -> created as a resource manager
   - Agent Badge (AGENT") -> created as a resource manager
   - Investor KYC Badge ("INVESTOR") -> created as a resource manager
   - Subscription Badge (SUBSCRIPTION") -> created as a resource manager


Export the component address and the badge addresses created on the instationation step. These will be displayed under "New Entities" in the output of the previous command. The owner badge can be found with their symbols when inspecting the default account and the platform badege when inspecting the component
   (`resim show $account`) & (`resim show $component`)

   ```sh
   export component=YOUR_COMPONENT_ADDRESS
   export security_manager_owner_badge=YOUR_SECURITIES_MANAGER_OWNER_BADGE_ADDRESS
   export platform_badge=YOUR_PLATFORM_BADGE_ADDRESS
   export issuer_badge=YOUR_ISSUER_BADGE_ADDRESS
   export agent_badge=YOUR_AGENT_BADGE_ADDRESS
   export investor_kyc_badge=YOUR_INVESTOR_KYC_BADGE_ADDRESS
   export investor_subscription_badge=YOUR_INVESTOR_SUBSCRIPTION_BADGE_ADDRESS

   ```
   ```sh
   export component=component_sim1crzj3axceqx6xz20uafqqeku636xf62hx0he9udl7t0pdfuse548z2
   export security_manager_owner_badge=resource_sim1tkwthfgnulwkxx9szkrkfm5dgq47uu57rumvpk53x2nqjct8lrm28q
   export platform_badge=resource_sim1t4zlcrpzd2ft0syk2xlvvgxtxwsf48v9n3j5t6fp557lz0ccqz5wr8
   export issuer_badge=resource_sim1ntgg55acup2q56fwrtq27typn4wygu5zryv5nrr6tcgvc3dg944fxm
   export agent_badge=resource_sim1nguh58s5slljyjye4uwqj3229w9ga8e75mnk2enx098p9adkn3uzf5
   export investor_kyc_badge=resource_sim1ng0mqpw3ufyl0xhu4x8yj7vacv4k5azmtv7wnmeqv0u9ytp3vzppzk
   export investor_subscription_badge=resource_sim1ngkepl5f6scvv9sg2ha27w6jwuwjg2n6tn85p4qnqp3ec204magtdw
   ```


2. The Security Manager Owner onboards an Issuer to the platform by minting the issuer badge.
   - Mint an issuer badge, depositing directly to the issuer account and inspecting it in that account.

   ```sh
   resim run manifests/02_owner_mint_issuer_badge.rtm
   resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
   resim show $issuer_account
   ```
   Make a note of the issuer badge local ID (including hashes). If you haven't changed the minting manifest it will be `#1#`.

   An alternative is to mint the badge directly to the Owner account and transfer it in a second step to the Issuer using 02a_owner_transfer_issuer_badge.rtm

   Once the badge is in the issuer account, it is essentially soulbound to that account

3. The Issuer is responsible for appointing an Agent to perform certain actions on their
   behalf
   - Mint an issuer agent badge from the issuer account, depositing to the issuer agent account, and inspecting it in the issuer agent account

   ```sh
   resim run manifests/03_issuer_mint_agent_badge.rtm
   resim set-default-account $agent_account $agent_privatekey $agent_account_badge
   resim show $agent_account
   ```
   Make a note of the agent badge local ID (including hashes). If you haven't changed the minting manifest it will be `#1#`.

4. The Issuer is responsible for initially creating the bond instrument. The instrument is
   represented as an NFT. The metadata describes the bond's characteristics or reference data. The individual NFT's represents the rights the owner of the security is entitled to e.g. coupons. The Issuer is responsible for defining the former and the Agent for the latter.

   Mint a bond instrument NFT from the issuer account to be added to the instrument manager vault.

   ```sh
   resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
   resim run manifests/04_issuer_create_instrument.rtm
   ```
   Make a note of the created resource in "New Entities" and export
   ```sh
   export bond_instrument=NEW_INSTRUMENT_RESOURCE_ADDRESS
   e.g.
   export bond_instrument=resource_sim1nfad085cjh4tlz64evxpwlwh974m5t3vcpmhuk8z0sdr2s0fw62g63
   ```

4. Optional - Update the metadata on the newly created bond instrument
   ```sh
   resim run manifests/04a_issuer_update_instrument_metadata.rtm
   ```
   Optional - Update the metadata on fields that are not allowed to be updated by the issuer
   ```sh
   resim run manifests/04b_issuer_update_instrument_metadata.rtm
   ```

5. Once the bond instrument has been set up, the next step is for the Issuer to open the
   subscription to allow investors to subscribe to the security. This calls the issuer_open_subscription method and sets the subscription_status field on the bond instrument metadata to "open"

   ```sh
   resim run manifests/05_issuer_open_subscription.rtm
   ```

6. The investor submits a kyc request, receives an investor kyc badge in return

   ```sh
   resim run manifests/06_investor_check_kyc.rtm
   resim set-default-account $investor_account $investor_privatekey $investor_account_badge
   resim show $investor_account
   ```
   Make a note of the agent badge local ID (including hashes). If you haven't changed the minting manifest it will be `#1#`.

7. Investor subscribes to the bond by presenting the KYC Badge, indicating the amount of
   securities to subscribe for and in return receives a subscription nft. The NFT data outlines the subscription terms, resource or bond to receive, amount of bond to receive and payment details required. It also includes a escrow_status field set to 'pending' initially and 'settled' once the investor transfers the funds or payment to a component vault

   ```sh
   resim run manifests/07_investor_subscribe.rtm
   resim show $investor_account
   ```

8. Investor is expected to transfer the payment resource to be held in escrow in a
   component vault up to the point the subscription period ends, at which point the fungible securities are minted and the investor can then withdraw the security.

   ```sh
   resim run manifests/08_investor_transfer.rtm
   resim show $investor_account
   ```

9. Optional - Investor is free to cancel the payment until the subscription period ends

   ```sh
   resim run manifests/09_investor_cancel_payment.rtm
   resim show $investor_account
    ```

   Investor can initiate a transfer again if desired as still in possession of the subscription NFT.
   ** For purposes of walking through this demo, a new transfer should be performed if the optional cancellation is performed

   ```sh
   resim run manifests/08_investor_transfer.rtm
   resim show $investor_account
   ```


10. Issuer closes the Subscription which updates the subscription_status field to "closed" 
   on the security

   ```sh
   resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
   resim show $issuer_account
   resim run manifests/09_issuer_close_subscription.rtm
   ```
   ** Investor cannot cancel the transfer once the subscription is closed. This could be relaxed but is in place as fits closely to a subscription process.

11. The Agent of the Issuer mints the nft's to describe the corporates actions or rights of
   the instrument. The first NFT is required to be of type "Issuance" followed by one or more "Coupons". For now the Issuer Agent mints the "Issuance" Lifecycle Event

   ```sh
   resim set-default-account $agent_account $agent_privatekey $agent_account_badge
   resim show $agent_account
   resim run manifests/11_agent_add_instrument_lifecycle_issuance.rtm
   resim show $component
   ```
   The minted NFT can be seen under "Owned Non-Fungible Resources" using the show $component


12. The Agent of the Issuer mints a fungible security for each corporate action assigned
   to the instrument - a new version for each corporate action or lifecycle event. As only the Issuance NFT exists, only v1 of the fungible security is minted at this point.

   ```sh
   resim set-default-account $agent_account $agent_privatekey $agent_account_badge
   resim show $agent_account
   resim run manifests/14_agent_issue_lifecycle_securities.rtm
   resim show $component
   ```
   The minted token can be seen under "Owned Fungible Resources" using the show $component

   Export the V1 of the Fungible Security as this will be used when claiming the coupon
   ```sh
   export bond_security_v1=NEW_SECURITY_RESOURCE_ADDRESS
   e.g.
   export bond_security_v1=resource_sim1t5mgxgr6aq4fh8kvycx9l4mkrf4axcp87060tywanhz7qqdpyx83ys
   ```

13. The investor can now claim the security marked for issuance and withdraw to their
   account. In this step, the investor returns the subscription badge to the component vault. The data on the returned badges will determine how much the issuer can withdraw as the issuance proceeeds

   ```sh
   resim set-default-account $investor_account $investor_privatekey $investor_account_badge
   resim run manifests/13_investor_claim_securities.rtm
   resim show $investor_account
   resim show $component
   ```

14. The issuer can claim the proceeds of the issuance once the investor has claimed
    the securities. The amount to be claimed is determined by the subscription badges returned to the component vault. Once the issuer claims the proceeds, the subscription badges in the vault are burned.

   ```sh
   resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
   resim show $issuer_account
   resim show $component
   resim run manifests/14_issuer_claim_cash.rtm
   resim show $issuer_account
   ```

15. The agent issuer adds the lifecycle events over the lifetime of the security by
    minting NFT's on the instrument and then issuing new versions of the fungible security. For now, we add the next 2 coupon's to the instrument in form of NFT'S

   ```sh
   resim set-default-account $agent_account $agent_privatekey $agent_account_badge
   resim show $agent_account
   resim run manifests/15_agent_add_instrument_lifecycle_coupons.rtm
   resim show $component
   ```
   The minted NFT's can be seen under "Owned Non-Fungible Resources" using the show $component

16. The Agent of the Issuer mints a fungible security for each corporate action assigned
   to the instrument. As 2 coupons are available on the instrument NFT, 2 new securities versions are created.

   ```sh
   resim set-default-account $agent_account $agent_privatekey $agent_account_badge
   resim show $agent_account
   resim run manifests/16_agent_issue_lifecycle_securities.rtm
   resim show $component
   ```
   The minted tokens can be seen under "Owned Fungible Resources" using the show $component
   ```sh

   Export the newly created security versions
   export bond_security_v2=NEW_SECURITY_RESOURCE_ADDRESS
   export bond_security_v3=NEW_SECURITY_RESOURCE_ADDRESS
   e.g.
   export bond_security_v2=resource_sim1t5023ykfxg2magqlw47drnwqf09tlxtpysfnt28kmr7zdpumefcqe4
   export bond_security_v3=resource_sim1thk70jt7ejfytkl08sf7fpk5ew75zqfnycr5rd4c53mqfzgtqfzn9n

17. As the issuer has withdrawn all the issuance proceeds, the issuer is required to transfer funds to the XRD component vault, else the coupon payments will fail

   ```sh
   resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
   resim show $issuer_account
   resim run manifests/17_issuer_transfer_funds.rtm
   resim show $issuer_account
   resim show $component
   ```

18. The investor can claim the coupons periodically. This is achieved by transferring
   the current version of the fungible securities, in return the coupoon payment is made and the next version of the securities is returned.

   ```sh
   resim set-default-account $investor_account $investor_privatekey $investor_account_badge
   resim run manifests/18_investor_claim_corporate_action.rtm
   resim show $investor_account
   resim show $component
   ```

   Only one coupon is processed at a time, the rtm can be rerun for next coupon by updating the security version in the manifest

   TAKE_FROM_WORKTOP
    Address("${bond_security_v1}")
    Decimal("100")
    Bucket("security_bucket")

## Consolidated Script
The provides a summary of all steps in the README.md file

```sh
source ./setup.sh

resim run manifests/01_instantiate_securities_manager.rtm
export component=component_sim1crzj3axceqx6xz20uafqqeku636xf62hx0he9udl7t0pdfuse548z2
export security_manager_owner_badge=resource_sim1tkwthfgnulwkxx9szkrkfm5dgq47uu57rumvpk53x2nqjct8lrm28q
export platform_badge=resource_sim1t4zlcrpzd2ft0syk2xlvvgxtxwsf48v9n3j5t6fp557lz0ccqz5wr8
export issuer_badge=resource_sim1ntgg55acup2q56fwrtq27typn4wygu5zryv5nrr6tcgvc3dg944fxm
export agent_badge=resource_sim1nguh58s5slljyjye4uwqj3229w9ga8e75mnk2enx098p9adkn3uzf5
export investor_kyc_badge=resource_sim1ng0mqpw3ufyl0xhu4x8yj7vacv4k5azmtv7wnmeqv0u9ytp3vzppzk
export investor_subscription_badge=resource_sim1ngkepl5f6scvv9sg2ha27w6jwuwjg2n6tn85p4qnqp3ec204magtdw

resim run manifests/02_owner_mint_issuer_badge.rtm
resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
resim show $issuer_account

resim run manifests/03_issuer_mint_agent_badge.rtm
resim set-default-account $agent_account $agent_privatekey $agent_account_badge
resim show $agent_account

resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
resim run manifests/04_issuer_create_instrument.rtm
export bond_instrument=resource_sim1nfad085cjh4tlz64evxpwlwh974m5t3vcpmhuk8z0sdr2s0fw62g63
resim run manifests/05_issuer_open_subscription.rtm

resim run manifests/06_investor_check_kyc.rtm
resim set-default-account $investor_account $investor_privatekey $investor_account_badge
resim show $investor_account
resim run manifests/07_investor_subscribe.rtm
resim show $investor_account

resim run manifests/08_investor_transfer.rtm
resim show $investor_account
resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
resim show $issuer_account
resim run manifests/10_issuer_close_subscription.rtm

resim set-default-account $agent_account $agent_privatekey $agent_account_badge
resim show $agent_account
resim run manifests/11_agent_add_instrument_lifecycle_issuance.rtm
resim show $component

resim set-default-account $agent_account $agent_privatekey $agent_account_badge
resim show $agent_account
resim run manifests/12_agent_issue_lifecycle_securities.rtm
resim show $component
export bond_security_v1=resource_sim1t5mgxgr6aq4fh8kvycx9l4mkrf4axcp87060tywanhz7qqdpyx83ys

resim set-default-account $investor_account $investor_privatekey $investor_account_badge
resim run manifests/13_investor_claim_securities.rtm
resim show $investor_account
resim show $component

resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
resim show $issuer_account
resim show $component
resim run manifests/14_issuer_claim_cash.rtm
resim show $issuer_account
export cash_vault=internal_vault_sim1tqc00uwvugml0xm0p4ccwke58lqvcxg8hwuefredwrvn26lsxnnnuh

resim set-default-account $agent_account $agent_privatekey $agent_account_badge
resim show $agent_account
resim run manifests/15_agent_add_instrument_lifecycle_coupons.rtm
resim show $component

resim set-default-account $agent_account $agent_privatekey $agent_account_badge
resim show $agent_account
resim run manifests/16_agent_issue_lifecycle_securities.rtm
resim show $component

export bond_security_v2=resource_sim1tkxqyfndhl8082jtc4wgrls9elpwvqzh6l7g0c2leg7vhk90p0up8r
export bond_security_v3=resource_sim1thgd2ud7m3849dd77gjltlax3282wmtykhgea2ht9cemc7jwc027ql

resim set-default-account $issuer_account $issuer_privatekey $issuer_account_badge
resim show $issuer_account
resim run manifests/17_issuer_transfer_funds.rtm
resim show $issuer_account
resim show $component

resim set-default-account $investor_account $investor_privatekey $investor_account_badge
resim run manifests/18_investor_claim_corporate_action.rtm
resim show $investor_account
resim show $component