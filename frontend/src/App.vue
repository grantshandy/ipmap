<template>
  <div>
  </div>
</template>

<script>
export default {
	name: "App",
  data() {
    return {
      eventSource: new EventSource("/ip_stream"),
      // eventData: new Array(),
    }
  },
  mounted() {
    this.eventSource.onmessage = (event) => {
      // this.eventData.push(event.data);
      
      let obj = JSON.parse(event.data);
      
      console.log(obj.city, obj.country);
    };
  },
}
</script>