<script lang="ts">
  import { navigate } from "svelte-native";
  import {
    generateErrorDialog,
    getUserData,
    invalidAddressAlert,
    stateAbbreviations,
  } from "~/lib/api";
  import {
    Address,
    UserType,
    login,
    createAccount,
    LoginResult,
    checkIfAddressFilledIn,
  } from "~/lib/api.ts";
  import Senior from "./Senior.svelte";

  import Splash from "./Splash.svelte";
  import Volunteer from "./Volunteer.svelte";

  export let isLogin: boolean;
  let headerText: String = isLogin ? "Login" : "Signup";

  let username: String = "Username",
    password: String = "Password",
    name: String = "Full Name",
    address: Address = {
      line1: "511 Kenwood Pkwy",
      line2: undefined,
      city: "Minneapolis",
      state: "Minnesota",
      zip: "55403",
    },
    userType: UserType = "Senior";
  let volunteerButton,
    seniorButton;
  const userTypeOptions = ["Senior", "Volunteer"];

  const loginOrSignup = async () => {
    if (isLogin) {
      const res: LoginResult = await login({
        username,
        password,
      });
      if (res == LoginResult.addressError) return invalidAddressAlert();
      if (res == LoginResult.usernameError)
        return generateErrorDialog("Username Not Found");
      if (res == LoginResult.passwordError)
        return generateErrorDialog("Password Incorrect");
      if (res == LoginResult.unknownError)
        return generateErrorDialog("Unknown Error. Please Try Again Later.");
      // const userData = await getUserData();
      if (userType == "Volunteer") return navigate({ page: Volunteer });
      if (userType == "Senior") return navigate({ page: Senior });
    } else {
      const res: LoginResult = await createAccount({
        username,
        name,
        address: checkIfAddressFilledIn(address),
        userType,
        password,
      });
      console.log(res);
      if (res == LoginResult.addressError) return invalidAddressAlert();
      if (res == LoginResult.usernameError)
        return generateErrorDialog("Username Already Exists");
      if (res == LoginResult.passwordError)
        return generateErrorDialog("Something Went Very Wrong");
      if (res == LoginResult.unknownError)
        return generateErrorDialog("Unknown Error. Please Try Again Later.");
      // const userData = await getUserData();
      console.log(userType);
      if (userType == "Volunteer") return navigate({ page: Volunteer });
      if (userType == "Senior") return navigate({ page: Senior });
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
            <textField
              bind:text={address.line1}
              hint="House Number & Street Address"
            />
          </stackLayout>
          <stackLayout>
            <label text="Address Line 2 (Optional)" />
            <textField
              bind:text={address.line2}
              hint="Apartment, Suite, Unit, Building, Floor"
            />
          </stackLayout>
          <flexboxLayout flexDirection="row" justifyContent="space-around">
            <stackLayout orientation="horizontal">
              <label text="City" />
              <textField bind:text={address.city} hint="City Name" />
            </stackLayout>
            <stackLayout orientation="horizontal">
              <label text="State" />
              <listPicker
                items={stateAbbreviations}
                bind:selectedValue={address.state}
                verticalAlignment="stretch"
                selectedIndex="23"
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
          <flexboxLayout>
            <button text="Senior" bind:this={seniorButton} on:tap={() => {userType = "Senior"}}></button>
            <button text="Volunteer" bind:this={volunteerButton} on:tap={() => {userType = "Volunteer"}}></button>
          </flexboxLayout>
          <!-- <listPicker
            items={userTypeOptions}
            bind:selectedValue={userType}
            verticalAlignment="stretch"
          /> -->
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
