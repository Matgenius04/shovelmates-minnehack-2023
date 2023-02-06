// for colored chalk support, run

import { readFile } from 'fs/promises'
import { argv } from 'process'
import { fileURLToPath } from 'url'
import { dirname, join } from 'path'
import chalk from 'chalk'

const serverURL = "http://0.0.0.0:8080"

// shows request data sent to api
const extraDebug = argv[2] == 'debug';
const showServerResponse = extraDebug || argv[2] == 'response';

const allUserInfoSenior = {
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
const allUserInfoVolunteer = {
  username: "exampleUser",
  password: "verysecure123",
  name: "Jane Doe",
  address: {
    line1: "1600 Amphitheatre Parkway",
    line2: undefined,
    city: "Mountain View",
    state: "CA",
    zip: "94043",
  },
  userType: "Volunteer"
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
const apiFetchPost = async (endpoint, body, statusErrors={}) => {
  const res = await fetch(`${serverURL}/api/${endpoint}`, {
    method: "POST",
    mode: "cors",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify(body)
  })
  const outText = await res.text();
  if (extraDebug) console.error(`\nData sent to ${endpoint}: ${chalk.yellow(JSON.stringify(body, null, 2))}\n`)
  if (showServerResponse) console.error(`Server response: ${chalk.cyanBright(outText)}`)
  if (!res.ok) {
    if (statusErrors[res.status]) throw statusErrors[res.status]
    throw res.status + " Error"
  }
  return {...res, text:()=>outText, json:()=>{try {return JSON.parse(outText)} catch {throw "Invalid JSON Server Response"}}};
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
  }, {
    '409': "username error"
  })
  authorizationString = await res.text()
  return res;
}
const login = async (user) => {
  const res = await apiFetchPost("login",{
    username: user.username, 
    password: user.password
  }, {
    '409': 'username error',
    '403': 'password error'
  })
  authorizationString = await res.text()
  return res.json();
}
const getUserData = async () => {
  const res = await apiFetchPost("user-data", {authorization: authorizationString})
  return res.json()
}
const requestHelp = async (helpRequest) => {
  const res = await apiFetchPost("request-help", {authorization: authorizationString, ...helpRequest}, {
    '405': "Not Senior Error"
  })
  return res.json()
}
const getSelfRequest = async () => {
  const res = await apiFetchPost(`help-requests`, {authorization: authorizationString}, {
    '409': "No Requests Exist Error"
  })
  return res.json()
}
const requestWork = async () => {
  const res = await apiFetchPost("request-work", {authorization: authorizationString}, {
    '405': "Not Volunteer Error"
  })
  return res.json()
}


const test = async (testFunction, testName, flip = false) => {
  try {
    await testFunction()
    if (flip) {
      console.error(chalk.red(`${testName} Test Failed`))
    } else {
      console.log(chalk.green(`${testName} Test Succeeded`))
    }
  } catch (e) {
    if (flip) {
      console.log(chalk.green(`${testName} Test Succeeded`))
    } else {
      console.error(`${chalk.red(`${testName} Test Failed:`)} ${e}`)
    }
  }
  if (extraDebug || showServerResponse) await console.log("-".repeat(40))
  else await null;
}



// TESTS
// create senior account test
await test((async ()=>{
  await createAccount(allUserInfoSenior)
  await console.log(`Authorization String: ${authorizationString}\n`)
}),"Senior Account Creation")

// create account duplication test
await test(createAccount.bind(this,allUserInfoSenior),"Duplicate Account Creation", true)

// login account test
await test(login.bind(this, allUserInfoSenior), "Login")

// get user data test
await test (getUserData, "Get User Data")

// request help test (as senior)
await test (requestHelp.bind(this, helpRequest), "Senior Help Request")

// get request made (as senior)
await test (getSelfRequest, "Senior Get Self Request")

if (!extraDebug && !showServerResponse) console.log()
authorizationString = ""
// create volunteer account test
await test((async ()=>{
  await createAccount(allUserInfoVolunteer)
  await console.log(`Authorization String: ${authorizationString}\n`)
}),"Volunteer Account Creation")
// get volunteer user data test
await test(getUserData, "Get User Data")
// request work (as volunteer)
await test(requestWork, "Volunteer Request Work")