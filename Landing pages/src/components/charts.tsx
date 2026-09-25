import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
} from "chart.js";
import { Bar } from "react-chartjs-2";

ChartJS.register(CategoryScale, LinearScale, BarElement);

/**
 * Small decorative bar chart used inside product mockups.
 * Renders from plain number arrays so it stays data-driven.
 */
export function MockBars({
  labels,
  values,
  highlightLast = true,
}: {
  labels: string[];
  values: number[];
  highlightLast?: boolean;
}) {
  const data = {
    labels,
    datasets: [
      {
        data: values,
        backgroundColor: values.map((_, i) =>
          highlightLast && i === values.length - 1 ? "#f97316" : "#e7e5e4",
        ),
        hoverBackgroundColor: "#fdba74",
        borderRadius: 3,
        borderSkipped: false as const,
        barPercentage: 0.74,
        categoryPercentage: 0.86,
      },
    ],
  };

  const options = {
    responsive: true,
    maintainAspectRatio: false,
    animation: { duration: 650, easing: "easeOutQuart" },
    plugins: { legend: { display: false }, tooltip: { enabled: false } },
    scales: {
      x: { display: false },
      y: { display: false, beginAtZero: true, grace: "8%" },
    },
  };

  return <Bar data={data} options={options as never} />;
}
