<script lang="ts">
  import * as camera from "@nativescript/camera";
  import { ImageSource } from "@nativescript/core";
  import {
    generateErrorDialog,
    generateSuccessDialog,
    HelpRequestResult,
    requestHelp,
    logout
  } from "~/lib/api";

  import { navigate } from "svelte-native";
  import Splash from "./Splash.svelte";

  let picture: string = "";
  let picturePreviewHidden = true;
  let pictureSource: ImageSource;
  let notes: string = "";

  const submitPicture = async () => {
    if (await camera.requestCameraPermissions()) {
      const takenPhoto = await camera.takePicture();
      picturePreviewHidden = false;
      pictureSource = await ImageSource.fromAsset(takenPhoto);
      picture = await pictureSource.toBase64StringAsync("jpg");
    }
  };
  const submit = async () => {
    console.log(picture)
    if (!picture) generateErrorDialog("Please submit a photo");
    try {
      const res = await requestHelp({
        picture,
        notes,
      });
      if (res == HelpRequestResult.success) generateSuccessDialog("Your request has been submitted");
    } catch (e) {
      console.error(e)
    }
  };
</script>

<page>
  <actionBar>
    <label text="Shovelmates" fontSize="25" />
    <button text="Logout" fontSize="25" on:tap={()=>{
      logout()
      navigate({page: Splash})
    }}></button>
  </actionBar>
  <flexboxLayout flexDirection="column" justifyContent="space-around">
    <stackLayout>
      <label text="Picture Submission"/>
      <image
        src={pictureSource}
        hidden={picturePreviewHidden}
        width="100%"
        height="250"
        stretch="aspectFill"
      />
      <button text="Take Snow Photo" on:tap={submitPicture} />
    </stackLayout>
    <stackLayout>
      <label text="Notes" />
      <textView
        bind:text={notes}
        hint="Parking details, specific times that don't work, etc."
      />
    </stackLayout>
    <button text="Submit" on:tap={submit} />
  </flexboxLayout>
</page>

<style>
  label {
    font-size: 16
  }
</style>
