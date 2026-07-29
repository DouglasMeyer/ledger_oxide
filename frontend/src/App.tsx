import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Provider } from "urql";
import { client } from "./lib/urql";
import Layout from "./components/Layout";
import AccountsPage from "./pages/AccountsPage";
import TransactionsPage from "./pages/TransactionsPage";
import ForecastPage from "./pages/ForecastPage";
import ImportPage from "./pages/ImportPage";
import ProjectionsPage from "./pages/ProjectionsPage";

export default function App() {
  return (
    <Provider value={client}>
      <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route index element={<Navigate to="/accounts" replace />} />
            <Route path="accounts" element={<AccountsPage />} />
            <Route path="transactions" element={<TransactionsPage />} />
            <Route path="forecast" element={<ForecastPage />} />
            <Route path="import" element={<ImportPage />} />
            <Route path="projections" element={<ProjectionsPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </Provider>
  );
}
