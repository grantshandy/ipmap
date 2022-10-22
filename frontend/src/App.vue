<template>
  <div>
    <h1>Data Received</h1>
    <ul>
    <li v-for="data in eventData">
      {{ data }}
    </li>
    </ul>
  </div>
</template>

<script>
export default {
	name: "App",
  data() {
    return {
      eventSource: new EventSource("/ip_stream"),
      eventData: new Array(),
    }
  },
  mounted() {
    this.eventSource.onmessage = (event) => {
      this.eventData.push(event.data);
    };
  },
}
</script>