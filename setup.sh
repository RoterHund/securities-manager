echo "Setting up Scrypto Environment and Package"

echo "\nResetting radix engine simulator..." 
resim reset

echo "\nCreating new account..."
temp_account=`resim new-account`
echo "$temp_account"
export account=`echo "$temp_account" | grep Account | grep -o "account_.*"`
export privatekey=`echo "$temp_account" | grep Private | sed "s/Private key: //"`
export account_badge=`echo "$temp_account" | grep Owner | grep -o "resource_.*"`
export xrd=`resim show $account | grep XRD | grep -o "resource_.\S*" | sed -e "s/://"`

echo "\nCreating new issuer account..."
temp_issuer_account=`resim new-account`
echo "$temp_issuer_account"
export issuer_account=`echo "$temp_issuer_account" | grep Account | grep -o "account_.*"`
export issuer_privatekey=`echo "$temp_issuer_account" | grep Private | sed "s/Private key: //"`
export issuer_account_badge=`echo "$temp_issuer_account" | grep Owner | grep -o "resource_.*"`

echo "\nCreating new issuer agent account..."
temp_agent_account=`resim new-account`
echo "$temp_agent_account"
export agent_account=`echo "$temp_agent_account" | grep Account | grep -o "account_.*"`
export agent_privatekey=`echo "$temp_agent_account" | grep Private | sed "s/Private key: //"`
export agent_account_badge=`echo "$temp_agent_account" | grep Owner | grep -o "resource_.*"`

echo "\nCreating new investor account..."
temp_investor_account=`resim new-account`
echo "$temp_investor_account"
export investor_account=`echo "$temp_investor_account" | grep Account | grep -o "account_.*"`
export investor_privatekey=`echo "$temp_investor_account" | grep Private | sed "s/Private key: //"`
export investor_account_badge=`echo "$temp_investor_account" | grep Owner | grep -o "resource_.*"`

echo "\nPublishing package..."
export package=`resim publish . | sed "s/Success! New Package: //"`

echo "\nSetup Complete"
echo "--------------------------"
echo "Environment variables set:"
echo "account = $account"
echo "privatekey = $privatekey"
echo "account_badge = $account_badge"
echo "xrd = $xrd"
echo "package = $package"
echo "--------------------------"
echo "Environment variables set for issuer:"
echo "issuer_account = $issuer_account"
echo "issuer_privatekey = $issuer_privatekey"
echo "issuer_account_badge = $issuer_account_badge"
echo "--------------------------"
echo "Environment variables set for issuer agent:"
echo "agent_account = $agent_account"
echo "agent_privatekey = $agent_privatekey"
echo "iagent_account_badge = $agent_account_badge"
echo "--------------------------"
echo "Environment variables set for investor:"
echo "investor_account = $investor_account"
echo "investor_privatekey = $investor_privatekey"
echo "investor_account_badge = $investor_account_badge"
