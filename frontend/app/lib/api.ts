import { ApplicationSettings } from "@nativescript/core";
import { Image } from "@nativescript/core";
import { navigate } from "svelte-native";
import { alert, confirm } from '@nativescript/core/ui/dialogs'

import Splash from "~/pages/Splash.svelte";

export const stateList: String[] = ["Alabama","Alaska","Arizona","Arkansas","California","Colorado","Connecticut","Delaware","District of Columbia","Florida","Georgia","Hawaii","Idaho","Illinois","Indiana","Iowa","Kansas","Kentucky","Louisiana","Maine","Maryland","Massachusetts","Michigan","Minnesota","Mississippi","Missouri","Montana","Nebraska","Nevada","New Hampshire","New Jersey","New Mexico","New York","North Carolina","North Dakota","Ohio","Oklahoma","Oregon","Pennsylvania","Rhode Island","South Carolina","South Dakota","Tennessee","Texas","Utah","Vermont","Virginia","Washington","West Virginia","Wisconsin","Wyoming"]
export type State = typeof stateList[number];
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
  picture: Image,
  notes: String,
  creationTime: Date,
  state: "Pending" | { AcceptedBy: User } | { MarkedCompletedBy: User }
}
export type HelpRequest = {
  picture: Image,
  notes: String
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
  picture: Image,
  notes: String,
  dist: Number,
  address: String
}

export const invalidAddressAlert = () => alert({
  title: "Invalid Address Input",
  message: "Missing Required Address Data"
})

// move to loginsignup page
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
  console.log(res.headers)
  console.log(await res.json());
  if (!res.ok) return LonLatRequestError.unknownError;
  const lonLat = (await res.json())?.features?.geometry?.coordinates
  if (typeof lonLat == undefined) return LonLatRequestError.unknownError;
  return lonLat as [Number, Number];
}

const askUserToCheckIfAddressIsCorrect = () => {
  
}

const getAuthorizationString = () : String | void => {
  const authorizationString = ApplicationSettings.getString("AuthorizationString")
  if (authorizationString == "") {
    navigate({page: Splash})
    return
  }
  return authorizationString
}
export const createAccount = async (user: UserSignup) : Promise<LoginResult> => {
  console.log(user);
  const location = await addressToLonLat(user.address)
  const addressString = concatAddress(user.address)
  if (location == LonLatRequestError.unknownError) return LoginResult.addressError;
  const res = await fetch("/api/create-account", {
    method: "POST",
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
  const res = await fetch("/api/login", {
    method: "POST",
    body: JSON.stringify(loginInfo)
  })
  if (res.status == 409) return LoginResult.usernameError;
  if (res.status == 403) return LoginResult.passwordError;
  if (res.ok) return LoginResult.success;
  return LoginResult.unknownError
}
export const requestHelp = async (helpRequest: HelpRequest) : Promise<HelpRequestResult> => {
  const res = await fetch("/request-help", {
    method: "POST",
    body: JSON.stringify({...helpRequest, authorization: getAuthorizationString()})
  })
  if (res.status == 405) return HelpRequestResult.notSenior
  if (!res.ok) return HelpRequestResult.unknownError
  return HelpRequestResult.success;
}
export const getSelfRequest = async () : Promise<SelfRequestError | SelfRequestResult> => {
  const res = await fetch("/help-requests", {
    method: "POST",
    body: JSON.stringify({authorization: getAuthorizationString()})
  })
  if (res.status == 409) return SelfRequestError.nonexistentError
  if (!res.ok) return SelfRequestError.unknownError
  return res.json();
}
export const requestWork = async () : Promise<WorkRequestsResult | WorkRequestError> => {
  const res = await fetch("/request-work", {
    method: "POST",
    body: JSON.stringify({authorization: getAuthorizationString()})
  })
  if (res.status == 405) return WorkRequestError.notVolunteer
  if (!res.ok) return WorkRequestError.unknownError
  return res.json();
}
export const getWorkRequestByID = async (id: WorkRequestByID) : Promise<WorkRequestByIDResult | WorkRequestError> => {
  const res = await fetch("/help-requests", {
    method: "POST",
    body: JSON.stringify({...id, authorization: getAuthorizationString()})
  })
  if (res.status == 405) return WorkRequestError.notVolunteer
  if (!res.ok) return WorkRequestError.unknownError
  return res.json();
}