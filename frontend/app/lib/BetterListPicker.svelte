<script lang="ts">
  // I FUPPASDIF HATE NATIVESCRIPT's super ass listpicker
  export let items: String[];

</script>

<page>
  <gridLayout style="background-color: rgba(0,0,0,0.8);">
    <listView items={items}>
    
    </listView>
  </gridLayout>
</page>