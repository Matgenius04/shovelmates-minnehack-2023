<script lang="ts">
    import { PropertyChangeData } from "@nativescript/core";
  import { navigate } from "svelte-native";
  import {
    Address,
    stateList,
    UserType,
    login,
    createAccount,
    concatAddress,
    checkIfAddressFilledIn
  } from "~/lib/api.ts";

  import Splash from "./Splash.svelte";

  export let isLogin: boolean;
  let headerText: String = isLogin ? "Login" : "Signup";

  let username: String,
    password: String,
    name: String,
    address: Address = {
      line1: undefined,
      line2: undefined,
      city: undefined,
      country: undefined,
      zip: undefined,
    },
    userType: UserType
  const userTypeOptions = ["Senior", "Volunteer"]

  const loginOrSignup = () => {
    if (isLogin) {
      login({
        username,
        password,
      });
    } else {
      createAccount({
        username,
        name,
        address: checkIfAddressFilledIn(address),
        userType,
        password
      });
    }
  };
</script>

<page>
  <actionBar title="Shovelmates">
    <flexboxLayout flexDirection="row" justifyContent="flex-start">
      <button
        text="Back to Splash"
        fontSize="25"
        on:tap={() => {
          navigate({ page: Splash });
        }}
      />
    </flexboxLayout>
  </actionBar>
  <flexboxLayout flexDirection="column" id="top-level-container">
    <flexboxLayout justifyContent="center" flexDirection="column" id="Header">
      <label text={headerText} fontSize="50" alignSelf="center" />
    </flexboxLayout>
    <flexboxLayout flexDirection="column">
      <stackLayout>
        <label text="Username" />
        <textField bind:text={username} hint="Username" />
      </stackLayout>
      <stackLayout>
        <label text="Password" />
        <textField bind:text={password} secure={true} hint="Password" />
      </stackLayout>
      {#if !isLogin}
        <stackLayout>
          <label text="Name" />
          <textField bind:text={name} hint="Full Name" />
        </stackLayout>
        <flexboxLayout flexDirection="column">
          <stackLayout>
            <label text="Address Line 1" />
            <textField bind:text={address.line1} hint="House Number & Street Address"/>
          </stackLayout>
          <stackLayout>
            <label text="Address Line 2 (Optional)" />
            <textField bind:text={address.line2} hint="Apartment, Suite, Unit, Building, Floor"/>
          </stackLayout>
          <flexboxLayout flexDirection="row" justifyContent="space-around">
            <stackLayout orientation="horizontal">
              <label text="City" />
              <textField bind:text={address.city} hint="City Name" />
            </stackLayout>
            <stackLayout orientation="horizontal">
              <label text="State" />
              <listPicker
                items={stateList}
                bind:selectedValue = {address.state}
                verticalAlignment = "stretch"
                selectedIndex = 0
              />
            </stackLayout>
          </flexboxLayout>
          <stackLayout>
            <label text="Zip / Postal Code" />
            <textField bind:text={address.zip} hint="Zip / Postal" />
          </stackLayout>
        </flexboxLayout>
        <stackLayout>
          <!-- Change text here to be less ew -->
          <label text="User Type" />
          <listPicker items={userTypeOptions} bind:selectedValue={userType} verticalAlignment = "stretch"></listPicker>
        </stackLayout>
      {/if}
      <button text="Submit" on:tap={loginOrSignup} />
    </flexboxLayout>
  </flexboxLayout>
</page>

<style>
  .top-level-container > * {
    margin-top: 5;
    margin-bottom: 5;
    padding-top: 5;
    padding-bottom: 5;
  }
  #Header {
    padding-bottom: 100px;
  }
  page label {
    font-size: 16;
  }
  stackLayout {
    orientation: horizontal;
    height: 30;
    width: 100;
  }
  stackLayout label {
    height: 30;
  }
  stackLayout textField {
    height: 50;
    width: 40%;
  }
  stackLayout listPicker {
    height: 60;
    width: 100%;
  }
</style>
