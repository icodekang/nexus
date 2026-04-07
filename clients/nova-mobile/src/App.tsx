import React from 'react'
import { NavigationContainer } from '@react-navigation/native'
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs'
import ChatScreen from './screens/ChatScreen'
import ModelSelectScreen from './screens/ModelSelectScreen'
import ProfileScreen from './screens/ProfileScreen'

const Tab = createBottomTabNavigator()

export default function App() {
  return (
    <NavigationContainer>
      <Tab.Navigator>
        <Tab.Screen name="Chat" component={ChatScreen} />
        <Tab.Screen name="Models" component={ModelSelectScreen} />
        <Tab.Screen name="Profile" component={ProfileScreen} />
      </Tab.Navigator>
    </NavigationContainer>
  )
}
