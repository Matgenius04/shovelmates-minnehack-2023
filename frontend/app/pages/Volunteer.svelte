<script lang="ts">
  import * as camera from "@nativescript/camera";
  import { ImageSource } from "@nativescript/core";
  import {
    generateErrorDialog,
    getWorkRequestByID,
    requestWork,
    WorkRequestByIDResult,
    WorkRequestError,
    WorkRequestsResult,
  } from "~/lib/api";

  import { Template } from 'svelte-native/components'

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
</script>

<page>
  <actionBar>
    <label text="Shovelmates" fontSize="25" />
  </actionBar>
  <flexboxLayout flexDirection="column">
    <label text="Volunteer Opportunities" fontSize="20" alignSelf="center">
      {#await workRequests then awaitedWorkRequests}
        <listView items={awaitedWorkRequests}>
          <Template let:item>
            <flexboxLayout flexDirection="row" width="90%" margin="auto">
              <label text={item.user.name} alignSelf="center"/>
              <image src={item.picture} alignSelf="center" stretch="aspectFit"/>
              <flexboxLayout flexWrap="wrap" width="50%">
                <label text={item.notes} />
              </flexboxLayout>
              <label text={item.dist.toString()} />
              <label text={item.address} />
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
