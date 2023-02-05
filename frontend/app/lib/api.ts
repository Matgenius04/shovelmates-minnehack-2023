import { ApplicationSettings } from "@nativescript/core";
import { Image, ImageAsset, ImageSource } from "@nativescript/core";
import { navigate } from "svelte-native";
import { alert, confirm } from '@nativescript/core/ui/dialogs'

import Splash from "~/pages/Splash.svelte";

const serverURL = "http://10.0.2.2:8080" // android
// const serverURL = "http://0.0.0.0:8080" // ios

export const stateList: Record<string, string> = {"AL":"Alabama","AK":"Alaska","AZ":"Arizona","AR":"Arkansas","CA":"California","CO":"Colorado","CT":"Connecticut","DE":"Delaware","FL":"Florida","GA":"Georgia","HI":"Hawaii","ID":"Idaho","IL":"Illinois","IN":"Indiana","IA":"Iowa","KS":"Kansas","KY":"Kentucky","LA":"Louisiana","ME":"Maine","MD":"Maryland","MA":"Massachusetts","MI":"Michigan","MN":"Minnesota","MS":"Mississippi","MO":"Missouri","MT":"Montana","NE":"Nebraska","NV":"Nevada","NH":"New Hampshire","NJ":"New Jersey","NM":"New Mexico","NY":"New York","NC":"North Carolina","ND":"North Dakota","OH":"Ohio","OK":"Oklahoma","OR":"Oregon","PA":"Pennsylvania","RI":"Rhode Island","SC":"South Carolina","SD":"South Dakota","TN":"Tennessee","TX":"Texas","UT":"Utah","VT":"Vermont","VA":"Virginia","WA":"Washington","WV":"West Virginia","WI":"Wisconsin","WY":"Wyoming"}
export const stateAbbreviations = Object.keys(stateList)
export type State = typeof stateAbbreviations[number]
export type UserType = "Senior" | "Volunteer";
export type User = {
  username: string,
  name: string
}
export type UserSignup = {
  username: String,
  name: String,
  address: FilledAddress,
  userType: UserType,
  password: String
}
export type UserData = {
  username: string,
  name: string,
  address: string,
  location: [number, number], // [lat, long]
  user_type: { Volunteer: string[] } | { Senior: string | null }, // Volunteer contains a list of every request ID they've accepted, Senior contains the request ID of the request they've made
}
export type Address = {
  line1: String | undefined,
  line2: String | undefined,
  city: String | undefined,
  state: State | undefined,
  zip: String | undefined
}
export type FilledAddress = {
  line1: String,
  line2: String,
  city: String,
  state: State,
  zip: String
}
export enum LonLatRequestError {
  unknownError
}
export type LoginParameters = {
  username: String,
  password: String
}
export enum LoginResult {
  success,
  addressError,
  usernameError,
  passwordError,
  unknownError
}
export enum HelpRequestResult {
  success,
  notSenior,
  unknownError
}
export enum SelfRequestError {
  nonexistentError,
  unknownError
}
export type SelfRequestResult = {
  picture: ImageSource,
  notes: String,
  creationTime: Date,
  state: "Pending" | { AcceptedBy: User } | { MarkedCompletedBy: User }
}
export type HelpRequest = {
  picture: string,
  notes: string
}
export type HelpRequestParsed = {
  picture: ImageSource,
  notes: string
}
export enum WorkRequestError {
  success,
  notVolunteer,
  unknownError
}
export type WorkRequestsResult = String[];
export type WorkRequestByID = {
  id: String
}
export type WorkRequestByIDResult = {
  user: User,
  picture: ImageSource,
  notes: string,
  dist: number,
  address: string
}

export const invalidAddressAlert = async () => {
  await alert({
  title: "Invalid Address Input",
  message: "Missing Required Address Data"
  })
  throw "Invalid Address"
}

// move some of the following to the loginsignup page
export const generateErrorDialog = (msg: string): void => {
  alert({
    title: "Error",
    message: msg
  });
}
export const generateSuccessDialog = (msg: string): void => {
  alert({
    title: "Success",
    message: msg
  });
}
export const concatAddress = async (address: FilledAddress) : Promise<String | void> => {
  return `${address.line1?.trim()}
${address.line2?.trim()}
${address.city?.trim()} ${address.state?.trim()} ${address.zip?.trim()}
United States of America`
}
export const checkIfAddressFilledIn = (address: Address) : FilledAddress | Promise<void> => {
  if (!address.city || !address.line1 || !address.state || !address.zip) {
    return invalidAddressAlert()
  }
  return address as FilledAddress;
}

const addressToLonLat = async (address: FilledAddress) : Promise<[Number, Number] | LonLatRequestError> => {
  console.log("running address -> lon lat request")
  
  const res = await fetch(
    // `https://nominatim.openstreetmap.org/search?street=${address.line1}&city=${address.city}&state=${address.state}&postalcode=${address.zip}&format=geocodejson&countrycodes=us`, 
    // `https://nominatim.openstreetmap.org/search?q=${address.line1}, ${address.city}, ${address.state}&format=geocodejson&countrycodes=us`, 
    `https://nominatim.openstreetmap.org/search?addressdetails=1&q=${address.line1}, ${address.city}, ${address.state}&format=jsonv2&countrycodes=us&limit=1`, 
    {
    method: "GET",
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    }
    // new URLSearchParams({
    //   street: address.line1,
    //   city: address.city,
    //   state: address.state,
    //   country: "The United States",
    //   postalcode: address.zip,
    //   format: "geocodejson",
    //   countrycodes: "us"
    // } as Record<symbol, String>)
  })
  console.log(`https://nominatim.openstreetmap.org/search?addressdetails=1&q=${address.line1}, ${address.city}, ${address.state}&format=jsonv2&countrycodes=us&limit=1`)
  if (!res.ok) return LonLatRequestError.unknownError;
  const json = await res.json()
  const queryData = json?.[0]
  const lonLat = [queryData?.lat, queryData?.lon]
  if (!lonLat[0] || !lonLat[1]) return LonLatRequestError.unknownError;
  console.log(lonLat)
  // @ts-expect-error
  return lonLat.map(v=>Number(v)) as [Number, Number];
}

// TODO: Implement
const askUserToCheckIfAddressIsCorrect = () => {
  
}

const parseRequestImage = async (json: {picture:string}): Promise<any> => {
  // @ts-expect-error
  json.picture = await ImageSource.fromBase64(json);
  return json;
}
const getAuthorizationString = () : string => {
  const authorizationString = ApplicationSettings.getString("AuthorizationString")
  console.log(authorizationString);
  let parsed: {expirationTime: Number};
  try {
    parsed = JSON.parse(authorizationString)
  } catch {
    throw "Inavalid authorization token";
  }
  if (Date.now() < parsed.expirationTime) {
    navigate({page: Splash})
    throw "Authorization Token Expired"
  }
  if (authorizationString == "") {
    navigate({page: Splash})
    throw "No Authorization String Found"
  }
  return authorizationString
}
export const createAccount = async (user: UserSignup) : Promise<LoginResult> => {
  console.log(user);
  const location = await addressToLonLat(user.address)
  const addressString = await concatAddress(user.address)
  if (location == LonLatRequestError.unknownError) return LoginResult.addressError;
  const res = await fetch(`${serverURL}/api/create-account`, {
    method: "POST",
    mode: "cors",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({
      username: user.username, 
      name: user.name,
      address: addressString,
      location,
      userType: user.userType,
      password: user.password
    })
  })
  if (res.status == 409) return LoginResult.usernameError;
  if (!res.ok) return LoginResult.unknownError;
  ApplicationSettings.setString("AuthorizationString", await res.text());
  return LoginResult.success;
}
export const login = async (loginInfo: LoginParameters) : Promise<LoginResult> => {
  const res = await fetch(`${serverURL}/api/login`, {
    method: "POST",
    mode: 'cors',
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify(loginInfo)
  })
  if (res.status == 409) return LoginResult.usernameError;
  if (res.status == 403) return LoginResult.passwordError;
  if (!res.ok) return LoginResult.unknownError;
  ApplicationSettings.setString("AuthorizationString", await res.text());
  return LoginResult.success
}
export const requestHelp = async (helpRequest: HelpRequest) : Promise<HelpRequestResult> => {
  await console.log({authorization: getAuthorizationString()})
  const res = await fetch(`${serverURL}/api/request-help`, {
    method: "POST",
    mode: 'cors',
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({authorization: getAuthorizationString(), ...helpRequest})
  })
  if (res.status == 405) return HelpRequestResult.notSenior
  if (!res.ok) return HelpRequestResult.unknownError
  return HelpRequestResult.success;
}
export const getSelfRequest = async () : Promise<SelfRequestError | SelfRequestResult> => {
  const res = await fetch(`${serverURL}/help-requests`, {
    method: "POST",
    mode: 'cors',
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({authorization: getAuthorizationString()})
  })
  if (res.status == 409) return SelfRequestError.nonexistentError
  if (!res.ok) return SelfRequestError.unknownError
  return parseRequestImage(await res.json());
}
export const requestWork = async () : Promise<WorkRequestsResult | WorkRequestError> => {
  const res = await fetch(`${serverURL}/request-work`, {
    method: "POST",
    mode: 'cors',
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({authorization: getAuthorizationString()})
  })
  if (res.status == 405) return WorkRequestError.notVolunteer
  if (!res.ok) return WorkRequestError.unknownError
  return res.json();
}
export const getWorkRequestByID = async (id: WorkRequestByID) : Promise<WorkRequestByIDResult | WorkRequestError> => {
  const res = await fetch(`${serverURL}/help-requests`, {
    method: "POST",
    mode: 'cors',
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({...id, authorization: getAuthorizationString()})
  })
  if (res.status == 405) return WorkRequestError.notVolunteer
  if (!res.ok) return WorkRequestError.unknownError
  return parseRequestImage(await res.json()) as Promise<WorkRequestByIDResult>;
}
export const getUserData = async (): Promise<UserData> => {
  console.log("bruh")
  const res = await fetch(`${serverURL}/api/user-data`, {
    method: "POST",
    mode: "cors",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({authorization: getAuthorizationString()})
  })
  console.log("bruh")
  if (!res.ok) throw "User data not found";
  return res.json()
}