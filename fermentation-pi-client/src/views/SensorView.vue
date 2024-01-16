<script setup lang="ts">
  import { Line } from 'vue-chartjs'
  import { Chart, registerables, type ChartOptions} from 'chart.js';
  import axios from 'axios';
import { onBeforeUnmount } from 'vue';
  
Chart.register(...registerables);

const initialData = await LoadInitialData();

let myChart: Chart;

  const options : ChartOptions<"line"> = {
    responsive: true,
    interaction: {
    mode: 'index',
    intersect: false,
    },
    plugins: {
    title: {
        display: true,
        text: 'Chart.js Line Chart - Multi Axis'
    }
    },
    scales: {
    y: {
        display: true,
        position: 'left',
    },
    y1: {
        display: true,
        position: 'right',

        // grid line settings
        grid: {
        drawOnChartArea: false, // only want the grid lines for one axis to show up
        },
    }
    }
};

let data = 
{
  labels: initialData.Labels,
  datasets: [
    {
      label: 'Dataset 1',
      data: initialData.Temperature,
      yAxisID: 'y',
    },
    {
      label: 'Dataset 2',
      data: initialData.Humidity,
      yAxisID: 'y1',
    }
  ]};

  async function LoadInitialData() {
    
    let sensorData = await axios.get('/sensor/historic/' + (Math.floor(Date.now() / 1000) - 60 * 60).toString() + '/' + Math.floor(Date.now() / 1000).toString() );
    return {
      Labels: sensorData.data.map((d: any) => new Date(d.time * 1000).toLocaleString()),
      Temperature: sensorData.data.map((d: any) => d.data.temp),
      Humidity: sensorData.data.map((d: any) => d.data.hum)    
    }
  }

  async function UpdateData() {
    let sensorData = await axios.get('./sensor/');

    data.labels.push(new Date().toLocaleString());
    data.datasets[0].data.push(sensorData.data.temp);
    data.datasets[1].data.push(sensorData.data.hum);
    myChart.update();
  }

  const update = setInterval(UpdateData, 1000);

  onBeforeUnmount(() => {
    clearInterval(update);
  });
</script>

<template>
  <main>
    <Line
      id="my-chart-id"
      :options="options"
      :data="data"
      :ref="(el:any) => { myChart = el?.chart }"
    />
  </main>
</template>