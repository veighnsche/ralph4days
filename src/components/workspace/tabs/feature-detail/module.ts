import { defineWorkspaceTabModule } from '../contracts'
import { FeatureDetailTabContent } from './content'
import { createFeatureDetailTab } from './factory'
import { parseFeatureDetailTabParams } from './schema'

export const featureDetailTabModule = defineWorkspaceTabModule({
  type: 'feature-detail',
  component: FeatureDetailTabContent,
  parseParams: parseFeatureDetailTabParams,
  createTab: createFeatureDetailTab
})
