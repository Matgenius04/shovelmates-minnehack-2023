<script lang="ts">
  import {navigate} from 'svelte-native'

  import Login from './LoginSignup.svelte'

  const navToLogin = () => {
    navigate({
      page: Login,
      props: {isLogin: true},
      // frame: "rootFrame"
    })
  }
  const navToSignup = () => {
    navigate({
      page: Login,
      props: {isLogin: false},
      // frame: "rootFrame"
    })
  }
</script>
<page>
  <actionBar>
    <label text="Shovelmates" fontSize="25"></label>
  </actionBar>
  <flexboxLayout justifyContent="space-around" flexDirection="column">
    <image src="~/assets/logo.png" width="80%" height="30%" stretch="aspectFit" alignSelf="center"></image>
    <button on:tap={navToLogin}>Login</button>
    <button on:tap={navToSignup}>Signup</button>
  </flexboxLayout>
</page>