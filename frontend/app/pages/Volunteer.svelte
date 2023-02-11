<script lang="ts">
  import * as camera from "@nativescript/camera";
  import { ImageSource } from "@nativescript/core";
  import {
    generateErrorDialog,
    getWorkRequestByID,
    logout,
    requestWork,
    WorkRequestByIDResult,
    WorkRequestError,
    WorkRequestsResult,
  } from "~/lib/api";

  import { Template } from 'svelte-native/components'
    import { navigate } from "svelte-native";
    import Splash from "./Splash.svelte";

  const getRequests = async (): Promise<WorkRequestByIDResult[]> => {
    const availableJobs = await requestWork();
    console.error(typeof availableJobs,availableJobs)
    if (typeof availableJobs == typeof WorkRequestError)
      generateErrorDialog("Invalid Request");
    const out = [];
    for (const id of availableJobs as WorkRequestsResult) {
      console.error("BRUHASDFa",id)
      const workRequest = getWorkRequestByID({ id });
      if (typeof workRequest == typeof Promise<WorkRequestByIDResult>)
        out.push((await workRequest) as WorkRequestByIDResult);
    }
    return out;
  };
  let workRequests = getRequests();
  (async ()=>{console.log(await getRequests())})()
</script>

<page>
  <actionBar>
    <label text="Shovelmates" fontSize="25" />
    <button text="Logout" fontSize="25" on:tap={()=>{
      logout()
      navigate({page: Splash})
    }}></button>
  </actionBar>
  <flexboxLayout flexDirection="column">
    <label text="Volunteer Opportunities" fontSize="20" alignSelf="center">
      {#await workRequests then awaitedWorkRequests}
        <listView items={awaitedWorkRequests}>
          <Template let:item={workRequest}>
            <flexboxLayout flexDirection="row" width="90%" margin="auto" style="background-color: blue;">
              <label text={workRequest.user.name} alignSelf="center"/>
              <image src={workRequest.picture} alignSelf="center" stretch="aspectFit"/>
              <flexboxLayout flexWrap="wrap" width="50%">
                <label text={workRequest.notes} />
              </flexboxLayout>
              <label text={workRequest.dist.toString()} />
              <label text={workRequest.address} />
              <button text="Accept Request"></button>
            </flexboxLayout>
          </Template>
        </listView>
      {/await}
    </label></flexboxLayout
  >
</page>

<style>
  label {
    font-size: 16
  }
</style>
