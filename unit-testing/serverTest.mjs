// for colored chalk support, run

import { readFile } from 'fs/promises'
import { fileURLToPath } from 'url'
import { dirname, join } from 'path'
import chalk from 'chalk'

const serverURL = "http://0.0.0.0:8080"
// const exampleUserCreateRequest = 

const allUserInfo = {
  username: "Username",
  password: "Password",
  name: "Full Name",
  address: {
    line1: "511 Kenwood Pkwy",
    line2: undefined,
    city: "Minneapolis",
    state: "MN",
    zip: "55403",
  },
  userType: "Senior"
}
const helpRequest = {
  picture: (await readFile(join(dirname(fileURLToPath(import.meta.url)),"../frontend/app/assets/logo.png"))).toString('base64url'),
  notes: "Example Notes here. TESTING testing 1234 boop bop bip bap"
}




const stateList = {"AL":"Alabama","AK":"Alaska","AZ":"Arizona","AR":"Arkansas","CA":"California","CO":"Colorado","CT":"Connecticut","DE":"Delaware","FL":"Florida","GA":"Georgia","HI":"Hawaii","ID":"Idaho","IL":"Illinois","IN":"Indiana","IA":"Iowa","KS":"Kansas","KY":"Kentucky","LA":"Louisiana","ME":"Maine","MD":"Maryland","MA":"Massachusetts","MI":"Michigan","MN":"Minnesota","MS":"Mississippi","MO":"Missouri","MT":"Montana","NE":"Nebraska","NV":"Nevada","NH":"New Hampshire","NJ":"New Jersey","NM":"New Mexico","NY":"New York","NC":"North Carolina","ND":"North Dakota","OH":"Ohio","OK":"Oklahoma","OR":"Oregon","PA":"Pennsylvania","RI":"Rhode Island","SC":"South Carolina","SD":"South Dakota","TN":"Tennessee","TX":"Texas","UT":"Utah","VT":"Vermont","VA":"Virginia","WA":"Washington","WV":"West Virginia","WI":"Wisconsin","WY":"Wyoming"}
// start of api implementation
let authorizationString;
const addressToLonLat = async (address) => {
  const res = await fetch(
    `https://nominatim.openstreetmap.org/search?addressdetails=1&q=${address.line1}, ${address.city}, ${stateList[address.state]}&format=jsonv2&countrycodes=us&limit=1`, 
    {
    method: "GET",
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    }
  })
  if (!res.ok) throw "Bad Request"
  const json = await res.json()
  const queryData = json?.[0]
  const lonLat = [queryData?.lat, queryData?.lon]
  if (!lonLat[0] || !lonLat[1]) throw "Uh oh. Address request didn't yield a lon lat....";
  return lonLat.map(v=>Number(v));
}
const apiFetchPost = async (endpoint, body) => {
  const res = await fetch(`${serverURL}/api/${endpoint}`, {
    method: "POST",
    mode: "cors",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify(body)
  })
  if (!res.ok) throw res.status + " Error"
  return res;
}
const concatAddress = async (address) => {
  return `${address.line1?.trim()}
${address.line2?.trim()}
${address.city?.trim()} ${address.state?.trim()} ${address.zip?.trim()}
United States of America`
}
const createAccount = async (user) => {
  const location = await addressToLonLat(user.address)
  const addressString = await concatAddress(user.address)
  const res = await apiFetchPost("create-account",{
      username: user.username, 
      name: user.name,
      address: addressString,
      location,
      userType: user.userType,
      password: user.password
  })
  authorizationString = await res.text()
  return res;
}
const login = async (user) => {
  const res = await apiFetchPost("login",{
    username: user.username, 
    password: user.password
  })
  authorizationString = await res.text()
  return res.json();
}
const getUserData = async () => {
  const res = await apiFetchPost("user-data", {authorization: authorizationString})
  if (!res.ok) throw "User data not found";
  return res.json()
}
const requestHelp = async (helpRequest) => {
  const res = await fetch("request-help", {authorization: authorizationString, ...helpRequest})
  if (res.status == 405) throw "Not Senior"
  if (!res.ok) throw "Unknown Error"
  return true;
}



// TESTS
// create account test
try {
  await createAccount(allUserInfo)
  console.log("Account Creation Succeeded")
  console.log(`\nAuthorization String: ${authorizationString}\n`)
} catch (e) {
  console.error(`Account Creation Failed: ${e}`)
}
try {
  await createAccount(allUserInfo)
  console.error(chalk.red("Duplicate Account Creation Test Failed"))
} catch (e) {
  console.log("Duplicate Account Creation Test Succeeded")
}
// login account test
try {
  await login(allUserInfo)
  console.log("Login Succeeded")
} catch (e) {
  console.error(`${chalk.red("Login Failed")}: ${e}`)
}
// get user data test
try {
  console.log(await getUserData(allUserInfo))
  console.log("User Data Request Succeeded")
} catch (e) {
  console.error(`${chalk.red("User Data Request Failed")}: ${e}`)
}
// request help test (as senior)
try {
  await requestHelp(helpRequest)
  console.log("Senior Help Request Succeeded")
} catch (e) {
  console.error(`${chalk.red("Senior Help Request Failed")}: ${e}`)
}
