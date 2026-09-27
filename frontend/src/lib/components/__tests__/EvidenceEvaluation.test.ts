import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/svelte';
import { vi, test, expect, beforeEach, afterEach } from 'vitest';
import EvidenceEvaluationStatus from '../EvidenceEvaluationStatus.svelte';
import EvidenceReviewPanel from '../EvidenceReviewPanel.svelte';

const mocks = vi.hoisted(() => ({ get: vi.fn(), post: vi.fn() }));
// Vitest's Node condition selects Svelte's SSR entry; use the real DOM lifecycle for mounted components.
vi.mock('svelte', () => vi.importActual<typeof import('svelte')>('svelte/internal'));
vi.mock('../../utils/api-client', () => ({ apiClient: mocks }));
vi.mock('../../convex/client', () => ({ isConvexEnabled: () => true }));
const source = 'Example Singer faced fraud charges. The court dismissed the charges on Tuesday.';
const job = { _id: 'job-1', sourceUrl: 'https://audit.invalid/article', sourceTitle: 'Court report',
  sourceText: source, status: 'no_support', model: 'jev-1.13.0',
  decision: { role: 'not_assessed', proceduralState: 'not_stated', categories: [], passage: null, reason: 'No supporting passage selected.' } };

beforeEach(() => { vi.resetAllMocks(); });
afterEach(() => { cleanup(); });

test('submission status distinguishes review from verification and can refresh after approval', async () => {
  mocks.get.mockResolvedValueOnce({ success: true, data: { status: 'needs_review' } });
  render(EvidenceEvaluationStatus, { jobId: 'job-1' });
  expect(await screen.findByText('Awaiting independent review')).toBeInTheDocument();
  expect(screen.getByText(/does not verify an allegation/)).toBeInTheDocument();
  mocks.get.mockResolvedValueOnce({ success: true, data: { status: 'approved' } });
  await fireEvent.click(screen.getByRole('button', { name: 'Refresh status' }));
  expect(await screen.findByText('Evidence approved by a reviewer')).toBeInTheDocument();
});

test('review requires a retained passage and sends the corrected procedural outcome', async () => {
  mocks.get.mockResolvedValue({ success: true, data: [job] });
  mocks.post.mockResolvedValue({ success: true });
  render(EvidenceReviewPanel, { artistId: 'artist-1' });
  await fireEvent.click(await screen.findByRole('button', { name: 'Review source' }));
  const approve = screen.getByRole('button', { name: 'Approve evidence' });
  expect(approve).toBeDisabled();
  await fireEvent.change(screen.getByLabelText('Category'), { target: { value: 'financial_crimes' } });
  await fireEvent.change(screen.getByLabelText('Procedural state reported by the source'), { target: { value: 'dismissed' } });
  await fireEvent.input(screen.getByLabelText('Review reason'), { target: { value: 'The court report explicitly records the dismissal.' } });
  await fireEvent.input(screen.getByLabelText(/Supporting passage/), { target: { value: 'This invented quotation is not present in the retained source.' } });
  expect(approve).toBeDisabled();
  await fireEvent.input(screen.getByLabelText(/Supporting passage/), { target: { value: source } });
  expect(approve).toBeEnabled();
  mocks.get.mockResolvedValue({ success: true, data: [] });
  await fireEvent.click(approve);
  await waitFor(() => expect(mocks.post).toHaveBeenCalledWith('/api/v1/evaluations/job-1/review', expect.objectContaining({
    approve: true, category: 'financial_crimes', proceduralState: 'dismissed', excerpt: source,
  })));
  await waitFor(() => expect(screen.queryByRole('region', { name: 'Evidence review queue' })).not.toBeInTheDocument());
});

test('failed review retains the form and displays the server error', async () => {
  mocks.get.mockResolvedValue({ success: true, data: [job] });
  mocks.post.mockResolvedValue({ success: false, message: 'An independent reviewer must review this submission.' });
  render(EvidenceReviewPanel, { artistId: 'artist-1' });
  await fireEvent.click(await screen.findByRole('button', { name: 'Review source' }));
  await fireEvent.input(screen.getByLabelText('Review reason'), { target: { value: 'This source does not substantiate the allegation.' } });
  await fireEvent.click(screen.getByRole('button', { name: 'Reject evidence' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('An independent reviewer');
  expect(screen.getByLabelText('Review reason')).toBeInTheDocument();
});

test('queue failures are visible even before any jobs load', async () => {
  mocks.get.mockResolvedValue({ success: false });
  render(EvidenceReviewPanel, { artistId: 'artist-1' });
  expect(await screen.findByRole('alert')).toHaveTextContent('Unable to load the review queue.');
});
