import { Compass } from 'lucide-react'
import { Link, Route, Routes } from 'react-router-dom'
import { AppShell } from './components/AppShell'
import { EmptyState } from './components/EmptyState'
import { CollectionDetailPage } from './features/collections/CollectionDetailPage'
import { CollectionsPage } from './features/collections/CollectionsPage'
import { EntryEditorPage } from './features/entries/EntryEditorPage'
import { EntryListPage } from './features/entries/EntryListPage'
import { SettingsPage } from './features/portability/SettingsPage'
import { SearchPage } from './features/search/SearchPage'
import { TagsPage } from './features/tags/TagsPage'
import { TrashPage } from './features/trash/TrashPage'

function NotFound() {
  return (
    <EmptyState
      icon={Compass}
      title="Page not found"
      hint="The route you followed does not exist."
      action={
        <Link to="/" className="btn btn-secondary">
          Back to all entries
        </Link>
      }
    />
  )
}

export default function App() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<EntryListPage />} />
        <Route path="entries/new" element={<EntryEditorPage />} />
        <Route path="entries/:id" element={<EntryListPage />} />
        <Route path="entries/:id/edit" element={<EntryEditorPage />} />
        <Route path="starred" element={<EntryListPage variant="starred" />} />
        <Route path="search" element={<SearchPage />} />
        <Route path="collections" element={<CollectionsPage />} />
        <Route path="collections/:slug" element={<CollectionDetailPage />} />
        <Route path="tags" element={<TagsPage />} />
        <Route path="trash" element={<TrashPage />} />
        <Route path="settings" element={<SettingsPage />} />
        <Route path="*" element={<NotFound />} />
      </Route>
    </Routes>
  )
}
